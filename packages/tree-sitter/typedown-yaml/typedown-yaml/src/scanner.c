/**
 * External scanner for typedown_yaml grammar
 */

#include "tree_sitter/parser.h"

#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

enum TokenType {
  NEWLINE,
  INDENT_MAPPING,
  INDENT_SEQUENCE,
  BLOCK_END,
  SEQ_ITEM_START,
  BLOCK_SCALAR_CONTENT,
};

enum IndentType {
  IND_ROOT = 0,
  IND_MAP = 1,
  IND_SEQ = 2,
};

#define MAX_INDENT_DEPTH 64
#define BLOCK_SCALAR_INDENT_UNSET UINT16_MAX

typedef struct {
  uint16_t indent_len[MAX_INDENT_DEPTH];
  uint8_t indent_typ[MAX_INDENT_DEPTH];
  uint16_t depth;
  uint16_t block_scalar_indent;
  bool in_block_scalar;
} Scanner;

void *tree_sitter_tdr_yaml_external_scanner_create(void) {
  return calloc(1, sizeof(Scanner));
}

void tree_sitter_tdr_yaml_external_scanner_destroy(void *payload) {
  free(payload);
}

unsigned tree_sitter_tdr_yaml_external_scanner_serialize(void *payload,
                                                         char *buffer) {
  Scanner *scanner = (Scanner *)payload;
  unsigned pos = 0;
  uint16_t count = scanner->depth + 1;
  unsigned needed = sizeof(uint16_t) + count * sizeof(uint16_t) +
                    count * sizeof(uint8_t) + sizeof(uint16_t) + sizeof(bool);
  if (needed > TREE_SITTER_SERIALIZATION_BUFFER_SIZE)
    return 0;
  memcpy(buffer + pos, &scanner->depth, sizeof(uint16_t));
  pos += sizeof(uint16_t);
  memcpy(buffer + pos, scanner->indent_len, count * sizeof(uint16_t));
  pos += count * sizeof(uint16_t);
  memcpy(buffer + pos, scanner->indent_typ, count * sizeof(uint8_t));
  pos += count * sizeof(uint8_t);
  memcpy(buffer + pos, &scanner->block_scalar_indent, sizeof(uint16_t));
  pos += sizeof(uint16_t);
  memcpy(buffer + pos, &scanner->in_block_scalar, sizeof(bool));
  pos += sizeof(bool);
  return pos;
}

void tree_sitter_tdr_yaml_external_scanner_deserialize(void *payload,
                                                       const char *buffer,
                                                       unsigned length) {
  Scanner *scanner = (Scanner *)payload;
  memset(scanner, 0, sizeof(Scanner));
  if (length == 0)
    return;

  unsigned pos = 0;
  if (pos + sizeof(uint16_t) > length)
    return;
  memcpy(&scanner->depth, buffer + pos, sizeof(uint16_t));
  pos += sizeof(uint16_t);

  if (scanner->depth >= MAX_INDENT_DEPTH) {
    scanner->depth = 0;
    return;
  }

  uint16_t count = scanner->depth + 1;
  if (pos + count * sizeof(uint16_t) > length)
    return;
  memcpy(scanner->indent_len, buffer + pos, count * sizeof(uint16_t));
  pos += count * sizeof(uint16_t);

  if (pos + count * sizeof(uint8_t) > length)
    return;
  memcpy(scanner->indent_typ, buffer + pos, count * sizeof(uint8_t));
  pos += count * sizeof(uint8_t);

  if (pos + sizeof(uint16_t) + sizeof(bool) <= length) {
    memcpy(&scanner->block_scalar_indent, buffer + pos, sizeof(uint16_t));
    pos += sizeof(uint16_t);
    memcpy(&scanner->in_block_scalar, buffer + pos, sizeof(bool));
  }
}

static bool at_newline(TSLexer *lexer) {
  return lexer->lookahead == '\n' || lexer->lookahead == '\r';
}

static void consume_newline(TSLexer *lexer) {
  if (lexer->lookahead == '\r')
    lexer->advance(lexer, false);
  if (lexer->lookahead == '\n')
    lexer->advance(lexer, false);
}

static uint16_t measure_indent(TSLexer *lexer) {
  uint16_t indent = 0;
  while (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
    indent++;
    lexer->advance(lexer, false);
  }
  return indent;
}

static void push_indent(Scanner *scanner, uint16_t col, uint8_t typ) {
  if (scanner->depth + 1 < MAX_INDENT_DEPTH) {
    scanner->depth++;
    scanner->indent_len[scanner->depth] = col;
    scanner->indent_typ[scanner->depth] = typ;
  }
}

bool tree_sitter_tdr_yaml_external_scanner_scan(void *payload, TSLexer *lexer,
                                                const bool *valid_symbols) {
  Scanner *scanner = (Scanner *)payload;

  uint16_t cur_ind = scanner->indent_len[scanner->depth];

  // Block scalar content: consume all indented lines after the indicator
  // The grammar handles | and > as internal tokens; the scanner handles content
  if (valid_symbols[BLOCK_SCALAR_CONTENT]) {
    if (!scanner->in_block_scalar) {
      scanner->in_block_scalar = true;
      scanner->block_scalar_indent = BLOCK_SCALAR_INDENT_UNSET;
    }

    // Skip trailing whitespace and newline after the indicator
    while (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
      lexer->advance(lexer, false);
    }
    if (at_newline(lexer)) {
      consume_newline(lexer);
    } else if (!lexer->eof(lexer)) {
      scanner->in_block_scalar = false;
      return false;
    }

    bool matched = false;

    while (!lexer->eof(lexer)) {
      uint16_t indent = measure_indent(lexer);

      // Blank line: always part of the block scalar
      if (at_newline(lexer)) {
        consume_newline(lexer);
        lexer->mark_end(lexer);
        matched = true;
        continue;
      }

      if (scanner->block_scalar_indent == BLOCK_SCALAR_INDENT_UNSET) {
        // First content line must be indented past the key
        if (indent == 0 && cur_ind == 0) {
          break;
        }
        scanner->block_scalar_indent = indent;
      }

      if (indent < scanner->block_scalar_indent) {
        break;
      }

      // Consume rest of line
      while (!lexer->eof(lexer) && !at_newline(lexer)) {
        lexer->advance(lexer, false);
      }
      lexer->mark_end(lexer);
      matched = true;

      // Consume newline to continue to next line
      if (at_newline(lexer)) {
        consume_newline(lexer);
      }
    }

    scanner->in_block_scalar = false;
    if (matched) {
      lexer->result_symbol = BLOCK_SCALAR_CONTENT;
      return true;
    }
    return false;
  }

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
        at_newline(lexer) || lexer->eof(lexer)) {
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

  if (!at_newline(lexer)) {
    return false;
  }

  // Mark before newline for zero-width _block_end
  lexer->mark_end(lexer);
  consume_newline(lexer);

  // Skip blank lines
  while (at_newline(lexer)) {
    consume_newline(lexer);
  }

  uint16_t indent = measure_indent(lexer);

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

  // Indent dropped: emit _block_end (zero-width and right before the newline)
  if (indent < cur_ind && valid_symbols[BLOCK_END] && scanner->depth > 0) {
    scanner->depth--;
    // Zero-width: don't mark_end, tree-sitter restores to before newline
    lexer->result_symbol = BLOCK_END;
    return true;
  }

  // Mark_end after all consumed content
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
