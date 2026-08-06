// Time formatting from epoch seconds

const MINUTE = 60;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;
const WEEK = 7 * DAY;
const MONTH = 30 * DAY;
const YEAR = 365 * DAY;

const TWO_WEEKS = 2 * WEEK;

export function formatAbsoluteTime (epochSecs: number): string {
  return new Date(epochSecs * 1000).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

export function formatEditTime (epochSecs: number, prefix: string): string {
  const delta = Math.floor(Date.now() / 1000) - epochSecs;

  if (delta < TWO_WEEKS) return `${prefix} ${formatRelativeTime(epochSecs)}`;

  return `${prefix} at ${formatAbsoluteTime(epochSecs)}`;
}

export function formatRelativeTime (epochSecs: number): string {
  const delta = Math.floor(Date.now() / 1000) - epochSecs;

  if (delta < MINUTE) return 'just now';
  if (delta < HOUR) return `${Math.floor(delta / MINUTE)}m ago`;
  if (delta < DAY) return `${Math.floor(delta / HOUR)}h ago`;
  if (delta < WEEK) return `${Math.floor(delta / DAY)}d ago`;
  if (delta < MONTH) return `${Math.floor(delta / WEEK)}w ago`;
  if (delta < YEAR) return `${Math.floor(delta / MONTH)}mo ago`;

  return `${Math.floor(delta / YEAR)}y ago`;
}

export function formatTime (epochSecs: number): string {
  const delta = Math.floor(Date.now() / 1000) - epochSecs;

  return delta < TWO_WEEKS ? formatRelativeTime(epochSecs) : formatAbsoluteTime(epochSecs);
}
