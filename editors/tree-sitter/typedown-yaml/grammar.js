/**
 * @file Tree-sitter grammar for typedown YAML frontmatter
 * @author Huy-DNA <huydo862003@gmail.com>
 * @license AGPL
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "typedown_yaml",

  rules: {
    // TODO: add the actual YAML grammar rules
    source_file: $ => "hello"
  }
});
