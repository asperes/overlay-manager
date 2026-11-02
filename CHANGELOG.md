# Changelog

## 0.1.0

First release.

- Set up games and the overlays that go with each. Pick the game's program, or choose it while it's running.
- Overlays start when the game starts (each with an optional delay) and close when it exits.
- Games and overlays that start through a launcher are tracked and closed correctly.
- Overlays running as administrator are closed behind a single UAC prompt; Windows system processes are never closed.
- Per overlay: on/off, close with the game, skip if already running. "Test overlays" starts them without the game.
- Shows each overlay's program name and icon, and flags programs that no longer exist.
- Deleting a game or overlay can be undone; changes save automatically.
- Lives in the system tray: tick games on or off, pause watching, or quit. Optional start with Windows.
- Light, dark and system themes.
