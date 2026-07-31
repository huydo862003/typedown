import type {
  ContentSummary,
} from '../types/content';

export function contentDisplayName (item: ContentSummary): string {
  return getResourceTitle(item.header, item.path);
}

export function contentHref (item: ContentSummary): string {
  return '/' + item.path.replace(/\.td$/, '');
}

// Resolve a display title from frontmatter _label, name, or the file path
export function getResourceTitle (header: Record<string, unknown>, path: string): string {
  if (header._label !== undefined) return String(header._label);
  if (header.name !== undefined) return String(header.name);

  const filename = path.replace(/\.td$/, '').split('/')
    .pop() ?? '';

  return unslugify(filename);
}

// Replace dashes and underscores with spaces, capitalize the first word
// e.g. "design-mockups" -> "Design mockups"
export function unslugify (slug: string): string {
  const words = slug.replace(/[-_]/g, ' ').split(' ');

  if (0 < words.length) {
    words[0] = words[0].charAt(0).toUpperCase() + words[0].slice(1);
  }

  return words.join(' ');
}
