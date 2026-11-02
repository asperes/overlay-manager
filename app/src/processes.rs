//! Starting overlays and stopping them again.

use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

/// How long overlays get to exit after being asked politely.
const GRACEFUL_TIMEOUT: Duration = Duration::from_secs(3);

/// Last segment of a Windows or POSIX path, for log messages.
pub fn file_name(path: &str) -> &str {
    path.rsplit(['\\', '/']).next().unwrap_or(path)
}

/// Starts a program and returns its PID, or 0 if Windows didn't give us one.
/// The working directory is the program's own folder, since many overlays load
/// files relative to it. Programs that need admin rights get the UAC prompt.
pub fn start(path: &str) -> io::Result<u32> {
    let path = path.trim().trim_matches('"');
    let mut cmd = Command::new(path);
    if let Some(dir) = Path::new(path).parent().filter(|d| !d.as_os_str().is_empty()) {
        cmd.current_dir(dir);
    }
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());

    match cmd.spawn() {
        #[cfg(unix)]
        Ok(mut child) => {
            let pid = child.id();
            // Reap the child when it exits so it doesn't linger as a zombie.
            std::thread::spawn(move || child.wait());
            Ok(pid)
        }
        #[cfg(not(unix))]
        Ok(child) => Ok(child.id()),
        #[cfg(windows)]
        Err(e) if e.raw_os_error() == Some(win::ERROR_ELEVATION_REQUIRED) => win::start_elevated(path),
        Err(e) => Err(e),
    }
}

/// Closes processes: asks politely, waits, then force-closes. If Windows
/// refuses (the process runs as admin) it asks for admin rights once, for all
/// of them together. Blocks for a few seconds; returns PIDs still running.
pub fn stop(pids: &[u32]) -> Vec<u32> {
    let own = std::process::id();
    let pids: Vec<u32> = pids.iter().copied().filter(|&pid| pid != own).collect();
    platform_stop(&pids)
}

#[cfg(windows)]
fn platform_stop(pids: &[u32]) -> Vec<u32> {
    use std::time::Instant;

    win::post_close(pids);
    let deadline = Instant::now() + GRACEFUL_TIMEOUT;
    let lingering: Vec<u32> = pids
        .iter()
        .copied()
        .filter(|&pid| !win::wait_for_exit(pid, deadline.saturating_duration_since(Instant::now())))
        .collect();

    let denied: Vec<u32> = lingering.into_iter().filter(|&pid| !win::terminate(pid)).collect();
    if denied.is_empty() {
        return Vec::new();
    }
    win::terminate_elevated(&denied);
    denied.into_iter().filter(|&pid| win::is_running(pid)).collect()
}

#[cfg(not(windows))]
fn platform_stop(pids: &[u32]) -> Vec<u32> {
    use std::time::Instant;

    let signal = |sig: &str, pid: u32| {
        Command::new("kill").args([sig, &pid.to_string()]).stderr(Stdio::null()).status().is_ok_and(|s| s.success())
    };
    for &pid in pids {
        signal("-TERM", pid);
    }
    let deadline = Instant::now() + GRACEFUL_TIMEOUT;
    let mut alive: Vec<u32> = pids.to_vec();
    while !alive.is_empty() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
        alive.retain(|&pid| signal("-0", pid));
    }
    for &pid in &alive {
        signal("-KILL", pid);
    }
    alive.retain(|&pid| signal("-0", pid));
    alive
}

#[cfg(windows)]
mod win {
    use std::ffi::OsStr;
    use std::io;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use std::ptr;
    use std::time::Duration;
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_ACCESS_DENIED, HANDLE, HWND, LPARAM, STILL_ACTIVE, WAIT_OBJECT_0,
    };
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, GetProcessId, OpenProcess, TerminateProcess, WaitForSingleObject,
        PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
    };
    use windows_sys::Win32::UI::Shell::{
        ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, PostMessageW, SW_HIDE, SW_SHOWNORMAL, WM_CLOSE,
    };
    use windows_sys::core::BOOL;

    pub const ERROR_ELEVATION_REQUIRED: i32 = 740;
    /// How long to wait for the elevated taskkill (includes the user answering UAC).
    const ELEVATED_TIMEOUT_MS: u32 = 60_000;

    fn wide(s: impl AsRef<OsStr>) -> Vec<u16> {
        s.as_ref().encode_wide().chain(Some(0)).collect()
    }

    /// Owns a process handle and closes it on drop.
    struct Process(HANDLE);

    impl Process {
        /// Opens `pid`. Err(true) means access was denied (process exists).
        fn open(pid: u32, access: u32) -> Result<Self, bool> {
            // SAFETY: plain Win32 call; the handle is owned by the returned value.
            let handle = unsafe { OpenProcess(access, 0, pid) };
            if handle.is_null() {
                Err(unsafe { GetLastError() } == ERROR_ACCESS_DENIED)
            } else {
                Ok(Self(handle))
            }
        }
    }

    impl Drop for Process {
        fn drop(&mut self) {
            // SAFETY: we own this handle.
            unsafe { CloseHandle(self.0) };
        }
    }

    unsafe extern "system" fn close_if_owned(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // SAFETY: lparam is the &[u32] passed by post_close, alive for the whole EnumWindows call.
        let pids = unsafe { &*(lparam as *const &[u32]) };
        let mut owner = 0u32;
        unsafe {
            GetWindowThreadProcessId(hwnd, &mut owner);
            if pids.contains(&owner) {
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
            }
        }
        1 // keep enumerating
    }

    /// Asks every top-level window of these processes to close.
    pub fn post_close(pids: &[u32]) {
        let pids: &[u32] = pids;
        // SAFETY: `pids` outlives the synchronous EnumWindows call.
        unsafe { EnumWindows(Some(close_if_owned), &pids as *const &[u32] as LPARAM) };
    }

    /// Waits for the process to exit. True if it's gone.
    pub fn wait_for_exit(pid: u32, timeout: Duration) -> bool {
        match Process::open(pid, PROCESS_SYNCHRONIZE) {
            // SAFETY: valid handle owned by `p`.
            Ok(p) => unsafe { WaitForSingleObject(p.0, timeout.as_millis() as u32) == WAIT_OBJECT_0 },
            Err(denied) => !denied,
        }
    }

    /// Force-closes the process. False if Windows denied access.
    pub fn terminate(pid: u32) -> bool {
        match Process::open(pid, PROCESS_TERMINATE) {
            // SAFETY: valid handle owned by `p`.
            Ok(p) => unsafe { TerminateProcess(p.0, 1) != 0 || !is_running(pid) },
            Err(denied) => !denied,
        }
    }

    pub fn is_running(pid: u32) -> bool {
        match Process::open(pid, PROCESS_QUERY_LIMITED_INFORMATION) {
            Ok(p) => {
                let mut code = 0u32;
                // SAFETY: valid handle owned by `p`; `code` is a valid out pointer.
                unsafe { GetExitCodeProcess(p.0, &mut code) != 0 && code == STILL_ACTIVE as u32 }
            }
            Err(denied) => denied,
        }
    }

    /// Runs `taskkill /F /T` for all PIDs as admin, behind a single UAC prompt.
    pub fn terminate_elevated(pids: &[u32]) {
        let args: Vec<String> = pids.iter().map(|pid| format!("/PID {pid}")).collect();
        let params = format!("/F /T {}", args.join(" "));
        let _ = shell_execute_runas("taskkill.exe", &params, None, SW_HIDE, Some(ELEVATED_TIMEOUT_MS));
    }

    /// Launches with the "runas" verb, which shows the UAC prompt.
    pub fn start_elevated(path: &str) -> io::Result<u32> {
        let dir = Path::new(path).parent().map(|d| d.as_os_str().to_owned());
        shell_execute_runas(path, "", dir.as_deref(), SW_SHOWNORMAL, None)
    }

    /// ShellExecuteEx with "runas". Returns the new PID (0 if unknown), after
    /// waiting up to `wait_ms` for it to finish when given.
    fn shell_execute_runas(
        file: impl AsRef<OsStr>,
        params: &str,
        dir: Option<&OsStr>,
        show: i32,
        wait_ms: Option<u32>,
    ) -> io::Result<u32> {
        let verb = wide("runas");
        let file = wide(file);
        let params = wide(params);
        let dir = dir.map(wide);

        // SAFETY: SHELLEXECUTEINFOW is a plain C struct; all-zero is a valid initial
        // state, and every string pointer outlives the ShellExecuteExW call.
        unsafe {
            let mut info: SHELLEXECUTEINFOW = std::mem::zeroed();
            info.cbSize = size_of::<SHELLEXECUTEINFOW>() as u32;
            info.fMask = SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC;
            info.lpVerb = verb.as_ptr();
            info.lpFile = file.as_ptr();
            info.lpParameters = params.as_ptr();
            info.lpDirectory = dir.as_ref().map_or(ptr::null(), |d| d.as_ptr());
            info.nShow = show;
            if ShellExecuteExW(&mut info) == 0 {
                return Err(io::Error::last_os_error());
            }
            if info.hProcess.is_null() {
                return Ok(0);
            }
            let process = Process(info.hProcess);
            if let Some(ms) = wait_ms {
                WaitForSingleObject(process.0, ms);
            }
            Ok(GetProcessId(process.0))
        }
    }
}
