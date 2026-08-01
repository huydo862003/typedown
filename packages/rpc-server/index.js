import { existsSync } from "node:fs";
import { execFileSync, spawn } from "node:child_process";
import { EventEmitter } from "node:events";
import { binPath } from "./platform.js";

function resolveBin() {
  // Prefer user's system binary
  try {
    const system = execFileSync("which", ["typedown-rpc"], { encoding: "utf-8" }).trim();
    if (system) return system;
  } catch {
    // ignore
  }

  // Fall back to downloaded binary
  const local = binPath();
  if (existsSync(local)) return local;

  throw new Error(
    `typedown-rpc binary not found. Run "pnpm install" to download it, ` +
      `install via Nix, or build manually with "cargo build --release -p typedown-server".`,
  );
}

const bin = resolveBin();

const DEFAULT_PORT = 4747;

export class RpcServer extends EventEmitter {
  constructor({ root, addr, port } = {}) {
    super();
    this._root = root ?? process.cwd();
    this._addr = addr ?? "127.0.0.1";
    this._port = port ?? DEFAULT_PORT;
    this._process = undefined;
    this._listening = false;
    this._resolvedAddress = undefined;
  }

  get address() {
    if (!this._listening) return undefined;
    return this._resolvedAddress;
  }

  get port() {
    if (!this._listening) return undefined;
    return Number(new URL(this._resolvedAddress).port);
  }

  get listening() {
    return this._listening;
  }

  listen(callback) {
    if (this._process) {
      throw new Error("Server is already running");
    }

    const child = spawn(bin, [], {
      cwd: this._root,
      env: {
        ...process.env,
        TYPEDOWN_RPC_ROOT: this._root,
        TYPEDOWN_RPC_ADDR: this._addr,
        TYPEDOWN_RPC_PORT: String(this._port),
      },
      stdio: ["ignore", "pipe", "inherit"],
    });

    this._process = child;

    const stdout = child.stdout;
    if (!stdout) {
      this.emit("error", new Error("Failed to capture typedown-rpc stdout"));
      return this;
    }

    // The server prints the ws:// address when ready
    stdout.once("data", (data) => {
      this._resolvedAddress = data.toString().trim();
      this._listening = true;
      this.emit("listening");
      if (callback) callback();
    });

    child.on("error", (error) => {
      this.emit("error", error);
    });

    child.on("exit", (code, signal) => {
      this._listening = false;
      this._process = undefined;
      this._resolvedAddress = undefined;
      this.emit("close", code, signal);
    });

    return this;
  }

  close() {
    if (this._process) {
      this._process.kill();
    }
    return this;
  }
}
