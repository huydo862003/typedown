// Polyfill for RegExp.escape (Stage 4, not yet in all runtimes)
export function escapeRegex (value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
