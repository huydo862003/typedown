/**
 * External scanner for typedown_yaml grammar
 */

#include "tree_sitter/parser.h"

#include <string.h>
#include <stdlib.h>
#include <stdbool.h>

enum TokenType {
  NEWLINE,
  INDENT_MAPPING,
  INDENT_SEQUENCE,
  BLOCK_END,
  SEQ_ITEM_START,
};

enum IndentType {
  IND_ROOT = 0,
  IND_MAP = 1,
  IND_SEQ = 2,
};

#define MAX_INDENT_DEPTH 64

typedef struct {
  uint16_t indent_len[MAX_INDENT_DEPTH];
  uint8_t indent_typ[MAX_INDENT_DEPTH];
  uint16_t depth;
} Scanner;

void *tree_sitter_typedown_yaml_external_scanner_create(void) {
  return calloc(1, sizeof(Scanner));
}

void tree_sitter_typedown_yaml_external_scanner_destroy(void *payload) {
  free(payload);
}

unsigned tree_sitter_typedown_yaml_external_scanner_serialize(void *payload,
                                                              char *buffer) {
  Scanner *scanner = (Scanner *)payload;
  unsigned pos = 0;
  memcpy(buffer + pos, &scanner->depth, sizeof(uint16_t));
  pos += sizeof(uint16_t);
  uint16_t count = scanner->depth + 1;
  memcpy(buffer + pos, scanner->indent_len, count * sizeof(uint16_t));
  pos += count * sizeof(uint16_t);
  memcpy(buffer + pos, scanner->indent_typ, count * sizeof(uint8_t));
  pos += count * sizeof(uint8_t);
  return pos;
}

void tree_sitter_typedown_yaml_external_scanner_deserialize(void *payload,
                                                            const char *buffer,
                                                            unsigned length) {
  Scanner *scanner = (Scanner *)payload;
  memset(scanner, 0, sizeof(Scanner));
  if (length == 0) return;

  unsigned pos = 0;
  if (pos + sizeof(uint16_t) > length) return;
  memcpy(&scanner->depth, buffer + pos, sizeof(uint16_t));
  pos += sizeof(uint16_t);

  if (scanner->depth >= MAX_INDENT_DEPTH) {
    scanner->depth = 0;
    return;
  }

  uint16_t count = scanner->depth + 1;
  if (pos + count * sizeof(uint16_t) > length) return;
  memcpy(scanner->indent_len, buffer + pos, count * sizeof(uint16_t));
  pos += count * sizeof(uint16_t);

  if (pos + count * sizeof(uint8_t) > length) return;
  memcpy(scanner->indent_typ, buffer + pos, count * sizeof(uint8_t));
}

static void push_indent(Scanner *scanner, uint16_t col, uint8_t typ) {
  if (scanner->depth + 1 < MAX_INDENT_DEPTH) {
    scanner->depth++;
    scanner->indent_len[scanner->depth] = col;
    scanner->indent_typ[scanner->depth] = typ;
  }
}

bool tree_sitter_typedown_yaml_external_scanner_scan(
    void *payload, TSLexer *lexer, const bool *valid_symbols) {
  Scanner *scanner = (Scanner *)payload;

  uint16_t cur_ind = scanner->indent_len[scanner->depth];

  // EOF: close blocks
  if (lexer->eof(lexer)) {
    if (valid_symbols[BLOCK_END] && scanner->depth > 0) {
      scanner->depth--;
      lexer->result_symbol = BLOCK_END;
      return true;
    }
    return false;
  }

  // - (sequence entry)
  if (valid_symbols[SEQ_ITEM_START] && lexer->lookahead == '-') {
    lexer->advance(lexer, false);
    if (lexer->lookahead == ' ' || lexer->lookahead == '\t' ||
        lexer->lookahead == '\n' || lexer->lookahead == '\r' ||
        lexer->eof(lexer)) {
      if (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
        lexer->advance(lexer, false);
      }
      lexer->mark_end(lexer);
      // Push content indent so _block_end fires at next -
      uint16_t content_col = cur_ind + 2;
      push_indent(scanner, content_col, IND_MAP);
      lexer->result_symbol = SEQ_ITEM_START;
      return true;
    }
  }

  // Newline: consume and measure next line's indent
  if (lexer->lookahead != '\n' && lexer->lookahead != '\r') {
    return false;
  }

  // Mark before newline for zero-width _block_end
  lexer->mark_end(lexer);

  if (lexer->lookahead == '\r') lexer->advance(lexer, false);
  if (lexer->lookahead == '\n') lexer->advance(lexer, false);

  // Skip blank lines
  while (lexer->lookahead == '\n' || lexer->lookahead == '\r') {
    if (lexer->lookahead == '\r') lexer->advance(lexer, false);
    if (lexer->lookahead == '\n') lexer->advance(lexer, false);
  }

  // Measure indent
  uint16_t indent = 0;
  while (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
    indent++;
    lexer->advance(lexer, false);
  }

  // EOF after newline
  if (lexer->eof(lexer)) {
    if (valid_symbols[BLOCK_END] && scanner->depth > 0) {
      scanner->depth--;
      lexer->result_symbol = BLOCK_END;
      return true;
    }
    if (valid_symbols[NEWLINE]) {
      lexer->mark_end(lexer);
      lexer->result_symbol = NEWLINE;
      return true;
    }
    return false;
  }

  // Indent dropped: emit _block_end (zero-width, before newline)
  if (indent < cur_ind && valid_symbols[BLOCK_END] && scanner->depth > 0) {
    scanner->depth--;
    // Zero-width: don't mark_end, tree-sitter restores to before newline
    lexer->result_symbol = BLOCK_END;
    return true;
  }

  // Mark_end after all consumed content (newline + indent)
  lexer->mark_end(lexer);

  // Deeper indent
  if (indent > cur_ind) {
    if (lexer->lookahead == '-' && valid_symbols[INDENT_SEQUENCE]) {
      push_indent(scanner, indent, IND_SEQ);
      lexer->result_symbol = INDENT_SEQUENCE;
      return true;
    }
    if (valid_symbols[INDENT_MAPPING]) {
      push_indent(scanner, indent, IND_MAP);
      lexer->result_symbol = INDENT_MAPPING;
      return true;
    }
  }

  // Same indent
  if (valid_symbols[NEWLINE]) {
    lexer->result_symbol = NEWLINE;
    return true;
  }

  return false;
}
