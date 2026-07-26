import { createRequire } from "node:module";
import path from "node:path";
import process from "node:process";
import dotenv from "dotenv";

const require = createRequire(import.meta.url);
const pkg = require("./package.json");

const env = dotenv.config({ path: new URL(".env", import.meta.url) });
const isDev = env.parsed?.DEV === "true";

export function dev() {
  return isDev;
}

const PLATFORM_MAP = {
  "linux-x64": "linux-x86_64",
  "darwin-x64": "darwin-x86_64",
  "darwin-arm64": "darwin-aarch64",
  "win32-x64": "windows-x86_64",
};

export function osArch() {
  const key = `${process.platform}-${process.arch}`;
  const mapped = PLATFORM_MAP[key];
  if (!mapped) {
    throw new Error(
      `Unsupported platform: ${process.platform} ${process.arch}`,
    );
  }
  return mapped;
}

export function releaseTag() {
  const version = pkg.version;
  if (version.includes("-")) {
    return `staging/v${version}`;
  }
  return `v${version}`;
}

export function artifactName(osArchStr) {
  const ext = process.platform === "win32" ? ".exe" : "";
  return `tdr-rpc-${pkg.version}-${osArchStr}${ext}`;
}

export function artifactUrl(tag, artifact) {
  return `https://github.com/Huy-DNA/typedown/releases/download/${tag}/${artifact}`;
}

export function repoRoot() {
  return path.resolve(path.dirname(import.meta.filename), "..", "..");
}

export function binPath() {
  const ext = process.platform === "win32" ? ".exe" : "";
  if (isDev) {
    return path.join(repoRoot(), "target", "debug", `tdr-rpc${ext}`);
  }
  return path.join(path.dirname(import.meta.filename), "bin", `tdr-rpc${ext}`);
}
