/*
* Copied from https://github.com/vuejs/vitepress/blob/main/src/node/markdown/plugins/preWrapper.ts
* Commit: 2fa0ded
* */

// Wraps fenced code blocks with a container div carrying:
// - Language label (<span class="lang">)
// - Copy button (<button class="copy">)

import type {
  MarkdownItAsync,
} from 'markdown-it-async';
import {
  extractLanguage,
} from './highlight/language';

export interface PreWrapperOptions {
  languageLabel?: Record<string, string>;
}

export function preWrapperPlugin (md: MarkdownItAsync, options: PreWrapperOptions = {}): void {
  // Build a case-insensitive lookup for custom language display names
  const languageLabel = Object.fromEntries(
    Object.entries(options.languageLabel || {})
      .map(([
        key,
        value,
      ]) => [
        key.toLowerCase(),
        value,
      ]),
  );

  const fence = md.renderer.rules.fence ?? md.renderer.renderToken.bind(md.renderer);

  md.renderer.rules.fence = function (...args): string {
    const [
      tokens,
      index,
    ] = args;
    const token = tokens[index];

    const language = extractLanguage(token.info);
    // Display underscores as spaces in the label (e.g. "my_lang" -> "my lang")
    const label = languageLabel[language.toLowerCase()] || language.replace(/_/g, ' ');

    return (
      `<div class="language-${language}">`
      + '<button title="Copy code" data-copied="Copied" class="copy"></button>'
      + `<span class="lang">${label}</span>`
      + fence(...args)
      + '</div>'
    );
  };
}
