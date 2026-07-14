/**
 * @file Tree-sitter host grammar for typedown
 * @author Huy-DNA <huydo862003@gmail.com>
 * @license AGPL
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "typedown",

  rules: {
    // TODO: add the actual grammar rules
    source_file: $ => "hello"
  }
});
