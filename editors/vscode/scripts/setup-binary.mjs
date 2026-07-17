// Sets up the tdr-lsp binary in bin/ based on TYPEDOWN_BINARY_SOURCE:
//   local   - copies from local build
//   staging - downloads the prerelease binary of the current version
import { copyFileSync, createWriteStream, mkdirSync, chmodSync, readFileSync, renameSync, rmSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { fileURLToPath } from 'node:url';
import { platform, arch, env } from 'node:process';

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const binName = platform === 'win32' ? 'tdr-lsp.exe' : 'tdr-lsp';
const binDir = join(repoRoot, 'editors', 'vscode', 'bin');
const binDest = join(binDir, binName);

const mode = env.TYPEDOWN_BINARY_SOURCE;

if (mode === 'local') {
  // local copy here

  mkdirSync(binDir, { recursive: true });
  const debugBinary = join(repoRoot, 'target', 'debug', binName);
  copyFileSync(debugBinary, binDest);
  console.log(`Copied ${debugBinary} -> ${binDest}`);
} else if (mode === 'staging') {
  // download binaries

  const version = readFileSync(join(repoRoot, 'VERSION'), 'utf8').trim();
  const releaseTag = version.includes('-') ? `staging/v${version}` : `v${version}`;

  // Artifact naming: tdr-lsp-{version}-{os}-{arch}[.exe]
  const osArchMap = {
    'linux-x64':    'linux-x86_64',
    'darwin-x64':   'darwin-x86_64',
    'darwin-arm64': 'darwin-aarch64',
    'win32-x64':    'windows-x86_64',
  };

  const platformKey = `${platform}-${arch === 'arm64' ? 'arm64' : 'x64'}`;
  const osArch = osArchMap[platformKey];
  if (!osArch) {
    console.error(`Unsupported platform: ${platformKey}`);
    process.exit(1);
  }
  const ext = platform === 'win32' ? '.exe' : '';
  const releaseArtifact = `tdr-lsp-${version}-${osArch}${ext}`;

  const downloadUrl = `https://github.com/huydo862003/typedown/releases/download/${encodeURIComponent(releaseTag)}/${releaseArtifact}`;
  console.log(`Downloading ${downloadUrl} -> ${binDest}`);

  const res = await fetch(downloadUrl);
  if (!res.ok) {
    console.error(`HTTP ${res.status}`);
    process.exit(1);
  }
  if (!res.body) {
    console.error('Response body is empty');
    process.exit(1);
  }

  mkdirSync(binDir, { recursive: true });
  const binTmp = `${binDest}.tmp`;

  try {
    await pipeline(
      Readable.fromWeb(res.body),
      createWriteStream(binTmp)
    );

    if (platform !== 'win32') chmodSync(binTmp, 0o755);
    renameSync(binTmp, binDest);
  } catch (err) {
    rmSync(binTmp, { force: true });
    throw err;
  }
  console.log('Done.');
} else {
  console.error(`TYPEDOWN_BINARY_SOURCE must be set to 'local' or 'staging' (got: ${mode ?? 'unset'})`);
  process.exit(1);
}
