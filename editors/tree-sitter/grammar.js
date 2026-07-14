/**
 * @file Tree-sitter grammar for typedown
 * @author Huy-DNA <huydo862003@gmail.com>
 * @license AGPL
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "typedown",

  rules: {
    // TODO: add the actual grammar rules
    source_file: $ => "hello"
  }
});
