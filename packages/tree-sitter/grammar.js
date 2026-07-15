/**
 * @file Typedown grammar for tree-sitter
 * @author Huy-DNA <huydo862003@gmail.com>
 * @license AGPL
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "typedown",

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
        "---\n",
        $.frontmatter_content,
        "---",
      ),

    body: ($) => $.body_content,
  },
});
