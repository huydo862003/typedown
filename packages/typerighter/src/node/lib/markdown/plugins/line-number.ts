/*
* Copied from https://github.com/vuejs/vitepress/blob/main/src/node/markdown/plugins/lineNumbers.ts
* Commit: 4e548f5
* */

// Depends on preWrapper plugin

import type {
  MarkdownItAsync,
} from 'markdown-it-async';

export function lineNumberPlugin (md: MarkdownItAsync, enable = false): void {
  const fence = md.renderer.rules.fence ?? md.renderer.renderToken.bind(md.renderer);

  md.renderer.rules.fence = function (...args): string {
    const rawCode = fence(...args);

    const [
      tokens,
      index,
    ] = args;
    const info = tokens[index].info;

    // `:line-numbers` / `:no-line-numbers` in the fence info overrides the global setting
    if (
      (!enable && !/:line-numbers\b/.test(info))
      || (enable && /:no-line-numbers\b/.test(info))
    ) {
      return rawCode;
    }

    // `=N` in the fence info sets the starting line number (e.g. `js =5`)
    let startLineNumber = 1;
    const matchStartLineNumber = info.match(/=(\d+)/);

    if (matchStartLineNumber && matchStartLineNumber[1]) {
      startLineNumber = parseInt(matchStartLineNumber[1]);
    }

    // Extract just the <code>...</code> portion to count lines
    const code = rawCode.slice(
      rawCode.indexOf('<code>'),
      rawCode.indexOf('</code>'),
    );

    const lines = code.split('\n');

    // Generate a <span> for each line number
    const lineNumbersCode = [...Array(lines.length)]
      .map(
        (_, index) =>
          `<span class="line-number">${index + startLineNumber}</span><br>`,
      )
      .join('');

    // Wrap in a hidden container
    const lineNumbersWrapperCode = `<div class="line-numbers-wrapper" aria-hidden="true">${lineNumbersCode}</div>`;

    // Inject the line numbers before the closing </div> and add the CSS class
    const finalCode = rawCode
      .replace(/<\/div>$/, `${lineNumbersWrapperCode}</div>`)
      .replace(/"(language-[^"]*?)"/, '"$1 line-numbers-mode"');

    return finalCode;
  };
}
