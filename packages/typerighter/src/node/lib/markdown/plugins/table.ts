/*
* Copied from https://github.com/vuejs/vitepress/blob/main/src/node/markdown/plugins/table.ts
* Commit: 2fa0ded
* */

// Adds tabindex="0" to tables so they are focusable and can be scrolled with the keyboard when they overflow horizontally

import type {
  MarkdownItAsync,
} from 'markdown-it-async';

export function tablePlugin (md: MarkdownItAsync): void {
  const tableOpen = md.renderer.rules.table_open;

  md.renderer.rules.table_open = function (tokens, index, options, env, self): string {
    const token = tokens[index];

    if (token.attrIndex('tabindex') < 0) {
      token.attrPush([
        'tabindex',
        '0',
      ]);
    }

    return tableOpen
      ? tableOpen(tokens, index, options, env, self)
      : self.renderToken(tokens, index, options);
  };
}
