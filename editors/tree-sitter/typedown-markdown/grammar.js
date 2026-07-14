/**
 * @file Tree-sitter grammar for typedown Markdown body
 * @author Huy-DNA <huydo862003@gmail.com>
 * @license AGPL
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "typedown_markdown",

  rules: {
    // TODO: add the actual Markdown grammar rules
    source_file: $ => "hello"
  }
});
