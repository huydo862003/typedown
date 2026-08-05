// Posix-style path utilities for content filepaths
// These work with forward-slash paths and run in both Node and browser

export function basename (filepath: string, extension?: string): string {
  const name = normalize(filepath).split('/')
    .pop() ?? '';

  if (extension !== undefined && name.endsWith(extension)) {
    return name.slice(0, -extension.length);
  }

  return name;
}

export function dirname (filepath: string): string {
  const parts = normalize(filepath).split('/');

  parts.pop();

  return parts.join('/');
}

export function extname (filepath: string): string {
  const name = basename(filepath);
  const dot = name.lastIndexOf('.');

  return dot <= 0 ? '' : name.slice(dot);
}

export function join (...segments: string[]): string {
  return segments
    .join('/')
    .replace(/\\+/g, '/')
    .replace(/\/+/g, '/')
    .replace(/\/$/, '') || '/';
}

function normalize (filepath: string): string {
  return filepath.replace(/\\/g, '/');
}
