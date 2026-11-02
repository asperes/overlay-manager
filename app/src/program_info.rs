//! Display name, icon and existence of a program, for showing overlays nicely.

use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramInfo {
    /// The exe's "File description" (e.g. "Discord"), or its file name without extension.
    pub name: String,
    /// PNG data URL of the program's icon, when it could be read.
    pub icon: Option<String>,
    pub exists: bool,
}

pub fn program_info(path: &str) -> ProgramInfo {
    let path = path.trim().trim_matches('"');
    let exists = Path::new(path).is_file();
    let stem = Path::new(&path.replace('\\', "/"))
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    #[cfg(windows)]
    if exists {
        return ProgramInfo {
            name: win::file_description(path).unwrap_or(stem),
            icon: win::icon_png(path).map(|png| {
                use base64::Engine;
                format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png))
            }),
            exists,
        };
    }

    ProgramInfo { name: stem, icon: None, exists }
}

#[cfg(windows)]
mod win {
    use std::ffi::{c_void, OsStr};
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::Graphics::Gdi::{
        DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        DIB_RGB_COLORS, HBITMAP,
    };
    use windows_sys::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
    use windows_sys::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};

    fn wide(s: impl AsRef<OsStr>) -> Vec<u16> {
        s.as_ref().encode_wide().chain(Some(0)).collect()
    }

    /// Reads `StringFileInfo\<lang>\FileDescription` from the exe's version resource.
    pub fn file_description(path: &str) -> Option<String> {
        let file = wide(path);
        // SAFETY: the buffer is sized by GetFileVersionInfoSizeW and outlives every
        // pointer VerQueryValueW returns into it.
        unsafe {
            let size = GetFileVersionInfoSizeW(file.as_ptr(), ptr::null_mut());
            if size == 0 {
                return None;
            }
            let mut data = vec![0u8; size as usize];
            if GetFileVersionInfoW(file.as_ptr(), 0, size, data.as_mut_ptr().cast()) == 0 {
                return None;
            }

            let mut ptr_out: *mut c_void = ptr::null_mut();
            let mut len = 0u32;
            let translation = wide(r"\VarFileInfo\Translation");
            if VerQueryValueW(data.as_ptr().cast(), translation.as_ptr(), &mut ptr_out, &mut len) == 0 || len < 4 {
                return None;
            }
            let lang = *(ptr_out as *const u16);
            let codepage = *(ptr_out as *const u16).add(1);

            let key = wide(format!(r"\StringFileInfo\{lang:04x}{codepage:04x}\FileDescription"));
            if VerQueryValueW(data.as_ptr().cast(), key.as_ptr(), &mut ptr_out, &mut len) == 0 || len == 0 {
                return None;
            }
            let chars = std::slice::from_raw_parts(ptr_out as *const u16, len as usize);
            let text = String::from_utf16_lossy(chars).trim_end_matches('\0').trim().to_string();
            (!text.is_empty()).then_some(text)
        }
    }

    /// The program's 32×32 shell icon, encoded as PNG.
    pub fn icon_png(path: &str) -> Option<Vec<u8>> {
        let file = wide(path);
        // SAFETY: every handle obtained here is released before returning.
        unsafe {
            let mut info: SHFILEINFOW = std::mem::zeroed();
            let ok = SHGetFileInfoW(
                file.as_ptr(),
                0,
                &mut info,
                size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON,
            );
            if ok == 0 || info.hIcon.is_null() {
                return None;
            }
            let mut icon: ICONINFO = std::mem::zeroed();
            let got_info = GetIconInfo(info.hIcon, &mut icon) != 0;
            DestroyIcon(info.hIcon);
            if !got_info {
                return None;
            }

            let result = icon_to_rgba(icon.hbmColor, icon.hbmMask)
                .and_then(|(w, h, rgba)| encode_png(w, h, &rgba));
            if !icon.hbmColor.is_null() {
                DeleteObject(icon.hbmColor);
            }
            DeleteObject(icon.hbmMask);
            result
        }
    }

    unsafe fn bitmap_bgra(bitmap: HBITMAP, width: i32, height: i32) -> Option<Vec<u8>> {
        let mut header: BITMAPINFO = unsafe { std::mem::zeroed() };
        header.bmiHeader = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // top-down rows
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..unsafe { std::mem::zeroed() }
        };
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        // SAFETY: `pixels` holds exactly width*height 32-bit pixels.
        let lines = unsafe {
            let dc = GetDC(ptr::null_mut());
            let lines = GetDIBits(
                dc,
                bitmap,
                0,
                height as u32,
                pixels.as_mut_ptr().cast(),
                &mut header,
                DIB_RGB_COLORS,
            );
            ReleaseDC(ptr::null_mut(), dc);
            lines
        };
        (lines == height).then_some(pixels)
    }

    unsafe fn icon_to_rgba(color: HBITMAP, mask: HBITMAP) -> Option<(u32, u32, Vec<u8>)> {
        if color.is_null() {
            return None; // monochrome icon; not worth handling
        }
        let mut bm: BITMAP = unsafe { std::mem::zeroed() };
        if unsafe { GetObjectW(color, size_of::<BITMAP>() as i32, (&mut bm as *mut BITMAP).cast()) } == 0 {
            return None;
        }
        let (w, h) = (bm.bmWidth, bm.bmHeight);
        let mut pixels = unsafe { bitmap_bgra(color, w, h) }?;

        // Old icons have no alpha channel: derive transparency from the AND mask instead.
        if pixels.chunks_exact(4).all(|p| p[3] == 0) {
            let mask_pixels = unsafe { bitmap_bgra(mask, w, h) }?;
            for (p, m) in pixels.chunks_exact_mut(4).zip(mask_pixels.chunks_exact(4)) {
                p[3] = if m[0] == 0 { 255 } else { 0 };
            }
        }
        for p in pixels.chunks_exact_mut(4) {
            p.swap(0, 2); // BGRA → RGBA
        }
        Some((w as u32, h as u32, pixels))
    }

    fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Option<Vec<u8>> {
        let mut out = Vec::new();
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header().ok()?.write_image_data(rgba).ok()?;
        Some(out)
    }
}
