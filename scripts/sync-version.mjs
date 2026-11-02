// Runs from `npm version`: copies the new package.json version into the Rust crate
// so Cargo.toml and Cargo.lock match the released app version.
import { readFileSync, writeFileSync } from "node:fs";

const version = JSON.parse(readFileSync("package.json", "utf8")).version;

const replace = (file, pattern) => {
  const text = readFileSync(file, "utf8");
  const next = text.replace(pattern, `$1${version}$2`);
  if (next === text && !text.includes(`"${version}"`)) throw new Error(`version not found in ${file}`);
  writeFileSync(file, next);
};

replace("app/Cargo.toml", /(^version = ")[^"]+(")/m);
replace("app/Cargo.lock", /(name = "overlay-manager"\nversion = ")[^"]+(")/);
console.log(`Synced version ${version}`);
