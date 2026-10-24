// Copies the legal/compliance files into the Windows bundler's SourceDir
// (src-tauri/target/release/legal/) so per-user-main.wxs can install them
// with an HKCU registry KeyPath component (bundler-generated resource
// components violate ICE38 for per-user installs — see tauri-bundler 2.10).
import { cpSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const dest = join(root, "src-tauri", "target", "release", "legal");
mkdirSync(dest, { recursive: true });

const files = [
  "LICENSE",
  "THIRD_PARTY_NOTICES.txt",
  "docs/legal/dependency-licenses.json",
  "docs/legal/supplemental-notices.json",
  "docs/legal/asset-provenance.json",
  "docs/legal/asset-license-notices.json",
  "docs/legal/third-party-license-audit.md",
  "docs/legal/dependency-follow-up.md",
  "docs/legal/dependency-follow-up.json",
  "docs/legal/brand-asset-sources.md",
];

for (const file of files) {
  cpSync(join(root, file), join(dest, file.split("/").pop()));
}
console.log(`copied ${files.length} legal files to ${dest}`);
