export function escapeHtml (value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('"', '&quot;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;');
}

export function stripHtml (html: string): string {
  return html.replace(/<[^>]*>/g, '').replace(/\s+/g, ' ')
    .trim();
}
