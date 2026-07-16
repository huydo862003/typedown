/**
 * @file Typedown TDR grammar for tree-sitter
 * @author Huy-DNA <huydo862003@gmail.com>
 * @license AGPL
 */

/* eslint-disable id-length */
/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: 'tdr',

  externals: ($) => [
    $.frontmatter_content,
    $.body_content,
  ],

  rules: {
    source_file: ($) =>
      seq(
        optional($.frontmatter),
        optional($.body),
      ),

    frontmatter: ($) =>
      seq(
        seq('---', /\r?\n/),
        $.frontmatter_content,
        '---',
      ),

    body: ($) => $.body_content,
  },
});
