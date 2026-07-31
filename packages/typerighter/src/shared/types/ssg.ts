import type {
  Component, Ref,
} from 'vue';
import type {
  MarkdownHeading,
} from './markdown';

export interface PageData {
  schema: string;
  frontmatter: Record<string, unknown>;
  headings: MarkdownHeading[];
  title: string;
}

export interface PageModule {
  __pageData: PageData;
  default: Component;
}

export interface TypedownData {
  /** Page-level data from the .td file */
  page: Ref<PageData>;
  /** Frontmatter fields */
  frontmatter: Ref<Record<string, unknown>>;
  /** Page title */
  title: Ref<string>;
  /** Dark mode state */
  isDark: Ref<boolean>;
}
