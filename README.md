# Overlay Manager

A small Windows tray app that starts your overlays when a game launches, and closes them when the game exits.

Pick a game, pick the overlays that go with it (Discord, RTSS, crosshair tools, …), and forget about it.

## Download

Get the installer from [Releases](https://github.com/asperes/overlay-manager/releases). The installer isn't code-signed yet, so Windows SmartScreen may warn on first run: choose **More info → Run anyway**.

## How it works

```
 ┌──────────────── Overlay Manager ────────────────┐
 │                                                 │
 │  every 2 s:  "which programs are running?"      │
 │      │        (the same list Task Manager shows)│
 │      ▼                                          │
 │  your game appeared?  → start its overlays      │
 │  your game gone?      → close those overlays    │
 │                                                 │
 └─────────────────────────────────────────────────┘
```

The project has two parts:

- **`app/`**: the program itself (Rust). It reads the list of running programs, decides when a game started or stopped, starts and closes overlays, and runs the tray icon.
- **`ui/`**: the settings window (React). It's only open while you're changing settings; closing it leaves Overlay Manager running in the tray.

A few details that make it reliable:

- **Launchers are handled.** Many games and overlays start through a launcher that opens the real program and exits. Overlay Manager follows the processes each one starts, so the real program is tracked and closed too.
- **Overlays close cleanly.** They're asked to close first, and only force-closed if they don't. Overlays running as administrator are closed behind a single UAC prompt.
- **It only closes what it opened.** Overlays you started yourself are left alone, and so are Windows system processes (Explorer, anything in `C:\Windows`, …).

## Why it won't affect your games

- **It never touches the game.** No injection, no reading or writing game memory, no hooks, no drivers, and it draws nothing on screen. It only reads the process list (name, file path, parent process), the same information Task Manager shows. That leaves anti-cheat systems nothing to react to. The overlays you choose are still your own call, though.
- **It never closes the game.** The only programs it closes are the overlays it started for you.
- **It's light.** Between checks it sleeps. A check is one read of the process list every 2 seconds (adjustable in Settings). There's no window open while you play, just the tray icon.

## Settings

Saved to `%APPDATA%\com.asperes.overlaymanager\config.json`.
