/**
 * External scanner for typedown_md_inline inline grammar
 */

#include "tree_sitter/parser.h"

#include <stdlib.h>
#include <stdbool.h>

enum TokenType {
  EMPHASIS_OPEN_STAR,
  EMPHASIS_CLOSE_STAR,
  EMPHASIS_OPEN_UNDERSCORE,
  EMPHASIS_CLOSE_UNDERSCORE,

  CODE_SPAN_DELIMITER,
  CODE_SPAN_CONTENT,

  MATH_SPAN_DELIMITER,
  MATH_SPAN_CONTENT,

  TEXT_CONTENT,
};

enum CharType {
  CHAR_WHITESPACE = 0,
  CHAR_PUNCTUATION = 1,
  CHAR_OTHER = 2,
};

typedef struct {
  uint8_t code_span_count;
  uint8_t math_span_count;
  uint8_t last_char_type;
} Scanner;

// Categorize character for emphasis flanking rules
static uint8_t char_type(int32_t character) {
  if (character == 0 || character == ' ' || character == '\t' ||
      character == '\n' || character == '\r') {
    return CHAR_WHITESPACE;
  }
  if ((character >= '!' && character <= '/') ||
      (character >= ':' && character <= '@') ||
      (character >= '[' && character <= '`') ||
      (character >= '{' && character <= '~')) {
    return CHAR_PUNCTUATION;
  }
  return CHAR_OTHER;
}

void *tree_sitter_typedown_md_inline_external_scanner_create(void) {
  return calloc(1, sizeof(Scanner));
}

void tree_sitter_typedown_md_inline_external_scanner_destroy(void *payload) {
  free(payload);
}

unsigned tree_sitter_typedown_md_inline_external_scanner_serialize(
    void *payload, char *buffer) {
  Scanner *scanner = (Scanner *)payload;
  buffer[0] = (char)scanner->code_span_count;
  buffer[1] = (char)scanner->math_span_count;
  buffer[2] = (char)scanner->last_char_type;
  return 3;
}

void tree_sitter_typedown_md_inline_external_scanner_deserialize(
    void *payload, const char *buffer, unsigned length) {
  Scanner *scanner = (Scanner *)payload;
  scanner->code_span_count = 0;
  scanner->math_span_count = 0;
  scanner->last_char_type = 0;
  if (length >= 3) {
    scanner->code_span_count = (uint8_t)buffer[0];
    scanner->math_span_count = (uint8_t)buffer[1];
    scanner->last_char_type = (uint8_t)buffer[2];
  }
}

// Match opening or closing span delimiter by count
static bool scan_span_delimiter(Scanner *scanner, TSLexer *lexer,
                                char delim_char, uint8_t *span_count,
                                int delim_symbol) {
  if (lexer->lookahead != delim_char) return false;

  // Mark_end before consuming so failed match is restorable
  lexer->mark_end(lexer);
  uint8_t count = 0;
  while (lexer->lookahead == delim_char) {
    count++;
    lexer->advance(lexer, false);
  }

  // ${ is interpolation, not math
  if (delim_char == '$' && count == 1 && lexer->lookahead == '{') {
    return false;
  }

  // Opening
  if (*span_count == 0) {
    *span_count = count;
    lexer->mark_end(lexer);
    lexer->result_symbol = delim_symbol;
    scanner->last_char_type = CHAR_PUNCTUATION;
    return true;
  }

  // Closing, count must match
  if (count == *span_count) {
    *span_count = 0;
    lexer->mark_end(lexer);
    lexer->result_symbol = delim_symbol;
    scanner->last_char_type = CHAR_PUNCTUATION;
    return true;
  }

  return false;
}

// Consume content between matching span delimiters
static bool scan_span_content(TSLexer *lexer, char delim_char,
                              uint8_t span_count, int content_symbol) {
  if (span_count == 0) return false;

  bool has_content = false;
  while (!lexer->eof(lexer)) {
    if (lexer->lookahead == delim_char) {
      // Mark before potential closing delimiter
      lexer->mark_end(lexer);
      uint8_t count = 0;
      while (lexer->lookahead == delim_char) {
        count++;
        lexer->advance(lexer, false);
      }
      if (count == span_count) {
        lexer->result_symbol = content_symbol;
        return has_content;
      }
      // Not a closing delimiter, delimiters become content
      has_content = true;
      continue;
    }
    lexer->advance(lexer, false);
    has_content = true;
  }

  if (has_content) {
    lexer->mark_end(lexer);
    lexer->result_symbol = content_symbol;
    return true;
  }
  return false;
}

// Classify * or _ as open, close, or plain text
static bool scan_emphasis(Scanner *scanner, TSLexer *lexer,
                          const bool *valid_symbols) {
  char delim_char;
  int open_token, close_token;

  if (lexer->lookahead == '*') {
    delim_char = '*';
    open_token = EMPHASIS_OPEN_STAR;
    close_token = EMPHASIS_CLOSE_STAR;
  } else if (lexer->lookahead == '_') {
    delim_char = '_';
    open_token = EMPHASIS_OPEN_UNDERSCORE;
    close_token = EMPHASIS_CLOSE_UNDERSCORE;
  } else {
    return false;
  }

  bool open_valid = valid_symbols[open_token];
  bool close_valid = valid_symbols[close_token];
  if (!open_valid && !close_valid && !valid_symbols[TEXT_CONTENT]) {
    return false;
  }

  uint8_t before = scanner->last_char_type;

  lexer->advance(lexer, false);
  lexer->mark_end(lexer);

  uint8_t after = char_type(lexer->lookahead);

  // Left-flanking: not followed by whitespace
  bool left_flanking =
      (after != CHAR_WHITESPACE) &&
      (after != CHAR_PUNCTUATION || before == CHAR_WHITESPACE ||
       before == CHAR_PUNCTUATION);
  // Right-flanking: not preceded by whitespace
  bool right_flanking =
      (before != CHAR_WHITESPACE) &&
      (before != CHAR_PUNCTUATION || after == CHAR_WHITESPACE ||
       after == CHAR_PUNCTUATION);

  // _ has stricter rules than *
  if (delim_char == '_') {
    bool can_open =
        left_flanking && (!right_flanking || before == CHAR_PUNCTUATION);
    bool can_close =
        right_flanking && (!left_flanking || after == CHAR_PUNCTUATION);

    if (can_close && close_valid) {
      lexer->result_symbol = close_token;
      scanner->last_char_type = CHAR_PUNCTUATION;
      return true;
    }
    if (can_open && open_valid) {
      lexer->result_symbol = open_token;
      scanner->last_char_type = CHAR_PUNCTUATION;
      return true;
    }
  } else {
    if (right_flanking && close_valid) {
      lexer->result_symbol = close_token;
      scanner->last_char_type = CHAR_PUNCTUATION;
      return true;
    }
    if (left_flanking && open_valid) {
      lexer->result_symbol = open_token;
      scanner->last_char_type = CHAR_PUNCTUATION;
      return true;
    }
  }

  // Emit as text so the character is not dropped
  if (valid_symbols[TEXT_CONTENT]) {
    lexer->result_symbol = TEXT_CONTENT;
    scanner->last_char_type = CHAR_PUNCTUATION;
    return true;
  }

  return false;
}

// Consume plain text, stopping at inline construct starters
static bool scan_text_content(Scanner *scanner, TSLexer *lexer,
                              const bool *valid_symbols) {
  if (!valid_symbols[TEXT_CONTENT]) return false;

  bool has_content = false;

  while (!lexer->eof(lexer)) {
    int32_t current = lexer->lookahead;

    if (current == '*' || current == '_' || current == '`' || current == '$' ||
        current == '[' || current == ']' || current == '!' || current == '\\' ||
        current == '{' || current == '}' || current == '(' || current == ')' ||
        current == '^' || current == '@') {
      break;
    }

    if (current == '\n' || current == '\r') break;

    scanner->last_char_type = char_type(current);
    lexer->advance(lexer, false);
    has_content = true;
  }

  if (has_content) {
    lexer->mark_end(lexer);
    lexer->result_symbol = TEXT_CONTENT;
    return true;
  }
  return false;
}

bool tree_sitter_typedown_md_inline_external_scanner_scan(
    void *payload, TSLexer *lexer, const bool *valid_symbols) {
  Scanner *scanner = (Scanner *)payload;

  // `
  if (scanner->code_span_count > 0) {
    if (valid_symbols[CODE_SPAN_DELIMITER] &&
        scan_span_delimiter(scanner, lexer, '`', &scanner->code_span_count,
                            CODE_SPAN_DELIMITER)) {
      return true;
    }
    if (valid_symbols[CODE_SPAN_CONTENT] &&
        scan_span_content(lexer, '`', scanner->code_span_count,
                          CODE_SPAN_CONTENT)) {
      return true;
    }
    return false;
  }

  // $
  if (scanner->math_span_count > 0) {
    if (valid_symbols[MATH_SPAN_DELIMITER] &&
        scan_span_delimiter(scanner, lexer, '$', &scanner->math_span_count,
                            MATH_SPAN_DELIMITER)) {
      return true;
    }
    if (valid_symbols[MATH_SPAN_CONTENT] &&
        scan_span_content(lexer, '$', scanner->math_span_count,
                          MATH_SPAN_CONTENT)) {
      return true;
    }
    return false;
  }

  // `
  if (lexer->lookahead == '`' && valid_symbols[CODE_SPAN_DELIMITER]) {
    if (scan_span_delimiter(scanner, lexer, '`', &scanner->code_span_count,
                            CODE_SPAN_DELIMITER)) {
      return true;
    }
  }

  // $
  if (lexer->lookahead == '$' && valid_symbols[MATH_SPAN_DELIMITER]) {
    if (scan_span_delimiter(scanner, lexer, '$', &scanner->math_span_count,
                            MATH_SPAN_DELIMITER)) {
      return true;
    }
  }

  // * _
  if (lexer->lookahead == '*' || lexer->lookahead == '_') {
    if (scan_emphasis(scanner, lexer, valid_symbols)) return true;
  }

  return scan_text_content(scanner, lexer, valid_symbols);
}
