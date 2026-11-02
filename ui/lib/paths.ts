/** Last segment of a Windows or POSIX path. */
export function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

/** Shortens a path to its last two parts ("…\Discord\Discord.exe") for display. */
export function shortPath(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  if (parts.length <= 2) return path;
  const sep = path.includes("\\") ? "\\" : "/";
  return `…${sep}${parts.slice(-2).join(sep)}`;
}

/** True when the value looks like a full path rather than a bare file name. */
export function isFullPath(path: string): boolean {
  return /[\\/]/.test(path);
}

/** Name to show for a game: its given name, else its exe, else a placeholder. */
export function gameTitle(game: { name: string; exe: string }): string {
  return game.name.trim() || (game.exe ? fileName(game.exe) : "New game");
}
