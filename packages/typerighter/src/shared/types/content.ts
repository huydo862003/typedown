export interface ContentSummary {
  /** Path relative to content dir, with extension */
  path: string;
  /** Schema type name */
  schema: string;
  /** Frontmatter header */
  header: Record<string, unknown>;
}

export type SidebarGroups = Record<string, ContentSummary[]>;
