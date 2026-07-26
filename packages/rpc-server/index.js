import { existsSync } from "node:fs";
import { binPath } from "./platform.js";

const bin = binPath();

if (!existsSync(bin)) {
  throw new Error(
    `tdr-rpc binary not found at ${bin}. Run "pnpm install" to download it, ` +
      `or build manually with "cargo build --release -p tdr-server".`,
  );
}

export const rpcBinaryPath = bin;
