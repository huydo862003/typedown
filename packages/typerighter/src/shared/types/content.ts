export interface ContentSummary {
  /** File path relative to content dir, with extension */
  filepath: string;
  /** Schema type name */
  schema: string;
  /** Frontmatter header */
  header: Record<string, unknown>;
}

export interface DirectoryEntry {
  name: string;
  url: string;
}

export type SchemaGroups = Record<string, ContentSummary[]>;

export interface ContentTreeNode {
  name: string;
  children: ContentTreeNode[];
  items: ContentSummary[];
}

export interface ContentTree {
  /** Files at the content root */
  rootItems: ContentSummary[];
  /** Top-level directory nodes */
  children: ContentTreeNode[];
}

export interface SubdirectoryEntry {
  name: string;
  url: string;
  count: number;
}

export interface DirectoryListing {
  title: string;
  url: string;
  subdirectories: SubdirectoryEntry[];
  items: DirectoryEntry[];
}
