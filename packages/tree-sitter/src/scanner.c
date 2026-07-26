#include "tree_sitter/parser.h"

enum TokenType {
  FRONTMATTER_CONTENT,
  BODY_CONTENT,
};

void *tree_sitter_typedown_external_scanner_create(void) { return NULL; }

void tree_sitter_typedown_external_scanner_destroy(void *payload) {}

unsigned tree_sitter_typedown_external_scanner_serialize(void *payload,
                                                         char *buffer) {
  return 0;
}

void tree_sitter_typedown_external_scanner_deserialize(void *payload,
                                                       const char *buffer,
                                                       unsigned length) {}

bool tree_sitter_typedown_external_scanner_scan(void *payload, TSLexer *lexer,
                                                const bool *valid_symbols) {
  if (valid_symbols[FRONTMATTER_CONTENT]) {
    // Scan until --- at line start
    bool at_line_start = true;
    while (!lexer->eof(lexer)) {
      if (at_line_start && lexer->lookahead == '-') {
        lexer->mark_end(lexer);
        lexer->advance(lexer, false);
        if (lexer->lookahead == '-') {
          lexer->advance(lexer, false);
          if (lexer->lookahead == '-') {
            // Found ---, stop before it
            lexer->result_symbol = FRONTMATTER_CONTENT;
            return true;
          }
        }
        at_line_start = false;
        continue;
      }
      at_line_start = (lexer->lookahead == '\n');
      lexer->advance(lexer, false);
    }
    return false;
  }

  if (valid_symbols[BODY_CONTENT]) {
    // Yield to grammar's --- literal if frontmatter not yet parsed
    if (!valid_symbols[FRONTMATTER_CONTENT]) {
      lexer->mark_end(lexer);
      while (lexer->lookahead == '\n' || lexer->lookahead == '\r' ||
             lexer->lookahead == ' ' || lexer->lookahead == '\t') {
        lexer->advance(lexer, true);
      }
      if (lexer->lookahead == '-') {
        lexer->advance(lexer, false);
        if (lexer->lookahead == '-') {
          lexer->advance(lexer, false);
          if (lexer->lookahead == '-') {
            return false;
          }
        }
        // Not ---, mark_end is before the dashes so they're included
      }
    }
    bool has_content = false;
    while (!lexer->eof(lexer)) {
      has_content = true;
      lexer->advance(lexer, false);
    }
    if (has_content) {
      lexer->mark_end(lexer);
      lexer->result_symbol = BODY_CONTENT;
      return true;
    }
    return false;
  }

  return false;
}
