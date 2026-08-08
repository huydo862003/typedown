/*
* Copied from https://github.com/vuejs/vitepress/blob/main/src/node/markdown/plugins/containers.ts
* Commit: b6d9cb8
* */

/* This file lets you create block-level custom containers in Markdown
* Example: ::: warning :::
* */

import {
  container,
} from '@mdit/plugin-container';
import type {
  MarkdownItAsync,
} from 'markdown-it-async';
import type {
  RenderRule,
} from 'markdown-it/lib/renderer.mjs';
import type StateBlock from 'markdown-it/lib/rules_block/state_block.mjs';

/** `type`, then optional `{props}`, then optional title */
const INFO_RE = /^\s*([a-zA-Z_]\w*)?\s*(\{[^}]*\})?\s*(.*)$/;
const PROP_RE = /([a-zA-Z_]\w*)(?:=(\d+|"[^"]*"|'[^']*'))?/g;
const SLOT_RE = /^={3}(?![=])[ \t]*([a-zA-Z_]\w*)?[ \t]*$/;

export function containerPlugin (
  md: MarkdownItAsync,
): void {
  md.use(container, {
    name: 'td',
    // Any identifier is a container type, so the only thing to reject is a
    // marker whose info string we cannot parse at all
    validate: (params) => INFO_RE.test(params),
    openRender: createOpenRender(md),
    closeRender: (tokens, index) =>
      (tokens[index].meta?.type === 'details' ? '</details>\n' : '</div>\n'),
  });

  // Typedown has no setext headings, so `===` under a paragraph is a slot separator, never an h1 underline
  md.block.ruler.disable('lheading');

  slotRule(md);
}

/** True when the tokens emitted so far leave a container open */
function isInsideContainer (state: StateBlock): boolean {
  let depth = 0;

  for (const token of state.tokens) {
    if (token.type === 'container_td_open') depth++;
    else if (token.type === 'container_td_close') depth--;
  }

  return 0 < depth;
}

/** `=== name` -- a flat separator token, as in the tree-sitter grammar */
function slotRule (md: MarkdownItAsync): void {
  md.block.ruler.before('fence', 'td_container_slot', (
    state,
    startLine,
    _endLine,
    silent,
  ) => {
    // Separators are only meaningful inside a container body. `parentType` is `'paragraph'` while terminator rules run, so count the open containers in the token stream instead
    if (!isInsideContainer(state)) return false;
    if (4 <= state.sCount[startLine] - state.blkIndent) return false;

    const start = state.bMarks[startLine] + state.tShift[startLine];
    const match = SLOT_RE.exec(state.src.slice(start, state.eMarks[startLine]));

    if (!match) return false;
    if (silent) return true;

    const token = state.push('container_slot_separator', 'div', 0);

    token.markup = '===';
    token.block = true;
    token.map = [
      startLine,
      startLine + 1,
    ];
    token.meta = {
      name: match[1] ?? null,
    };

    state.line = startLine + 1;

    return true;
  }, {
    alt: [
      'paragraph',
      'reference',
      'blockquote',
      'list',
    ],
  });

  // TODO: placeholder markup until the rendering shape is decided
  md.renderer.rules.container_slot_separator = (tokens, index) => {
    const name = tokens[index].meta.name;

    return name === null
      ? '<hr class="custom-block-slot">\n'
      : `<hr class="custom-block-slot" data-slot-name="${md.utils.escapeHtml(name)}">\n`;
  };
}

const DEFAULT_TITLES: Record<string, string> = {
  tip: 'TIP',
  info: 'INFO',
  warning: 'WARNING',
  danger: 'DANGER',
  details: 'Details',
  note: 'NOTE',
  important: 'IMPORTANT',
  caution: 'CAUTION',
};

function createOpenRender (
  md: MarkdownItAsync,
): RenderRule {
  return (tokens, index) => {
    const token = tokens[index];

    // `::: warning {closable} Don't do this`
    //   -> type `warning`, props `{closable}`, title `Don't do this`
    const [
      ,
      type = '',
      rawProps,
      info,
    ] = INFO_RE.exec(token.info) ?? [];

    // Stashed for the close render and for downstream consumers
    token.meta = {
      type,
      props: parseProps(rawProps),
    };

    // Build HTML attributes (e.g. `class="warning custom-block"`)
    token.attrJoin('class', `${type} custom-block`.trim());
    const renderedAttrs = md.renderer.renderAttrs(token).trim();

    // TODO: unknown types get no default title. Decide what they should emit
    if (!info && !(type in DEFAULT_TITLES))
      return `<div ${renderedAttrs}>\n`;

    // Render the title as inline markdown so e.g. **bold** works in titles
    const title = md.renderInline(info || DEFAULT_TITLES[type]);

    // ::: details renders as <details><summary>
    if (type === 'details')
      return `<details ${renderedAttrs}><summary>${title}</summary>\n`;
    // When the user provides a custom title, omit the `-default` class
    const titleClass =
      'custom-block-title' + (info ? '' : ' custom-block-title-default');

    return `<div ${renderedAttrs}><p class="${titleClass}">${title}</p>\n`;
  };
}

/** `{key=value flag}` -> `{ key: value, flag: true }` */
function parseProps (raw: string | undefined): Record<string, string | number | true> {
  const props: Record<string, string | number | true> = {};

  if (!raw) return props;

  for (const [
    ,
    key,
    value,
  ] of raw.slice(1, -1).matchAll(PROP_RE)) {
    if (value === undefined) props[key] = true;
    else if (/^\d+$/.test(value)) props[key] = Number(value);
    else props[key] = value.slice(1, -1);
  }

  return props;
}
