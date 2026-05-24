#[cfg(test)]
mod tests {
  use std::cell::RefCell;
  use std::rc::Rc;

  use typedown_types::string_stream::StringStream;

  use crate::green::cache::Cache;
  use crate::lex::ctx::{LexCtx, LexMode};
  use typedown_types::syntax_kind::SyntaxKind;

  fn lex_yaml(input: &str) -> Vec<(SyntaxKind, String)> {
    let stream = StringStream::new(input);
    let cache = Rc::new(RefCell::new(Cache::new()));
    let mut lexer = LexCtx::new(stream, cache);
    let mut tokens = vec![];
    loop {
      let result = lexer.lex();
      let kind = result.token.kind();
      let text: String = result.token.text().collect();
      tokens.push((kind, text));
      if kind == SyntaxKind::Eof {
        break;
      }
    }
    tokens
  }

  fn lex_markdown(input: &str) -> Vec<(SyntaxKind, String)> {
    let stream = StringStream::new(input);
    let cache = Rc::new(RefCell::new(Cache::new()));
    let mut lexer = LexCtx::new(stream, cache);
    lexer.set_mode(LexMode::MarkdownBody);
    let mut tokens = vec![];
    loop {
      let result = lexer.lex();
      let kind = result.token.kind();
      let text: String = result.token.text().collect();
      tokens.push((kind, text));
      if kind == SyntaxKind::Eof {
        break;
      }
    }
    tokens
  }

  /* YAML mode tests */

  #[test]
  fn yaml_triple_dash() {
    let tokens = lex_yaml("---");
    assert_eq!(tokens[0], (SyntaxKind::YamlOp, "---".to_string()));
  }

  #[test]
  fn yaml_colon() {
    let tokens = lex_yaml(":");
    assert_eq!(tokens[0], (SyntaxKind::Colon, ":".to_string()));
  }

  #[test]
  fn yaml_ident() {
    let tokens = lex_yaml("hello_world");
    assert_eq!(tokens[0], (SyntaxKind::Ident, "hello_world".to_string()));
  }

  #[test]
  fn yaml_number_integer() {
    let tokens = lex_yaml("42");
    assert_eq!(tokens[0], (SyntaxKind::Number, "42".to_string()));
  }

  #[test]
  fn yaml_number_decimal() {
    let tokens = lex_yaml("3.14");
    assert_eq!(tokens[0], (SyntaxKind::Number, "3.14".to_string()));
  }

  #[test]
  fn yaml_number_scientific() {
    let tokens = lex_yaml("2.5e10");
    assert_eq!(tokens[0], (SyntaxKind::Number, "2.5e10".to_string()));
  }

  #[test]
  fn yaml_number_trailing_dot() {
    let tokens = lex_yaml("1.");
    assert_eq!(tokens[0], (SyntaxKind::Number, "1.".to_string()));
  }

  #[test]
  fn yaml_number_missing_exponent_digits() {
    let tokens = lex_yaml("2.5E+");
    assert_eq!(tokens[0].0, SyntaxKind::Error);
  }

  #[test]
  fn yaml_dq_string() {
    let tokens = lex_yaml("\"hello\"");
    assert_eq!(tokens[0], (SyntaxKind::DqStrStart, "\"".to_string()));
    assert_eq!(tokens[1], (SyntaxKind::DqStrContent, "hello".to_string()));
    assert_eq!(tokens[2], (SyntaxKind::DqStrEnd, "\"".to_string()));
  }

  #[test]
  fn yaml_sq_string() {
    let tokens = lex_yaml("'hello'");
    assert_eq!(tokens[0], (SyntaxKind::SqStrStart, "'".to_string()));
    assert_eq!(tokens[1], (SyntaxKind::SqStrContent, "hello".to_string()));
    assert_eq!(tokens[2], (SyntaxKind::SqStrEnd, "'".to_string()));
  }

  #[test]
  fn yaml_empty_string() {
    let tokens = lex_yaml("\"\"");
    assert_eq!(tokens[0], (SyntaxKind::DqStrStart, "\"".to_string()));
    assert_eq!(tokens[1], (SyntaxKind::DqStrEnd, "\"".to_string()));
  }

  #[test]
  fn yaml_string_with_escape() {
    let tokens = lex_yaml("\"he\\\"llo\"");
    assert_eq!(tokens[0], (SyntaxKind::DqStrStart, "\"".to_string()));
    assert_eq!(
      tokens[1],
      (SyntaxKind::DqStrContent, "he\\\"llo".to_string())
    );
    assert_eq!(tokens[2], (SyntaxKind::DqStrEnd, "\"".to_string()));
  }

  #[test]
  fn yaml_unterminated_string() {
    let tokens = lex_yaml("\"hello\n");
    assert_eq!(tokens[0], (SyntaxKind::DqStrStart, "\"".to_string()));
    assert_eq!(tokens[1].0, SyntaxKind::Error);
  }

  #[test]
  fn yaml_comment() {
    let tokens = lex_yaml("# this is a comment");
    assert_eq!(
      tokens[0],
      (SyntaxKind::YamlComment, "# this is a comment".to_string())
    );
  }

  #[test]
  fn yaml_indent() {
    // Leading space on a non-empty line emits YamlIndent
    let tokens = lex_yaml(" a");
    assert_eq!(tokens[0].0, SyntaxKind::YamlIndent);
    assert_eq!(tokens[1], (SyntaxKind::Ident, "a".to_string()));
  }

  #[test]
  fn yaml_empty_line_no_indent() {
    // Leading space on an empty line (before EOF) emits Whitespace, not YamlIndent
    let tokens = lex_yaml(" ");
    assert_eq!(tokens[0], (SyntaxKind::Whitespace, " ".to_string()));
  }

  #[test]
  fn yaml_whitespace_after_token() {
    // Whitespace after a token emits Whitespace
    let tokens = lex_yaml("a b");
    assert_eq!(tokens[0], (SyntaxKind::Ident, "a".to_string()));
    assert_eq!(tokens[1], (SyntaxKind::Whitespace, " ".to_string()));
    assert_eq!(tokens[2], (SyntaxKind::Ident, "b".to_string()));
  }

  #[test]
  fn yaml_newline() {
    let tokens = lex_yaml("\n");
    assert_eq!(tokens[0], (SyntaxKind::Newline, "\n".to_string()));
  }

  #[test]
  fn yaml_crlf() {
    let tokens = lex_yaml("\r\n");
    assert_eq!(tokens[0], (SyntaxKind::Newline, "\r\n".to_string()));
  }

  #[test]
  fn yaml_bang_op() {
    // ! is an op char, but followed by alpha it stops (alpha is not an op char)
    let tokens = lex_yaml("!string");
    assert_eq!(tokens[0], (SyntaxKind::YamlOp, "!".to_string()));
    assert_eq!(tokens[1], (SyntaxKind::Ident, "string".to_string()));
  }

  #[test]
  fn yaml_bang_equals_op() {
    let tokens = lex_yaml("!=");
    assert_eq!(tokens[0], (SyntaxKind::YamlOp, "!=".to_string()));
  }

  #[test]
  fn yaml_arrow_op() {
    let tokens = lex_yaml("->");
    assert_eq!(tokens[0], (SyntaxKind::YamlOp, "->".to_string()));
  }

  #[test]
  fn yaml_brackets() {
    let tokens = lex_yaml("[]{}(),");
    assert_eq!(tokens[0], (SyntaxKind::LBracket, "[".to_string()));
    assert_eq!(tokens[1], (SyntaxKind::RBracket, "]".to_string()));
    assert_eq!(tokens[2], (SyntaxKind::LBrace, "{".to_string()));
    assert_eq!(tokens[3], (SyntaxKind::RBrace, "}".to_string()));
    assert_eq!(tokens[4], (SyntaxKind::LParen, "(".to_string()));
    assert_eq!(tokens[5], (SyntaxKind::RParen, ")".to_string()));
    assert_eq!(tokens[6], (SyntaxKind::Comma, ",".to_string()));
  }

  #[test]
  fn yaml_interp_in_string() {
    let tokens = lex_yaml("\"hello ${name}\"");
    assert_eq!(tokens[0], (SyntaxKind::DqStrStart, "\"".to_string()));
    assert_eq!(tokens[1], (SyntaxKind::DqStrContent, "hello ".to_string()));
    assert_eq!(tokens[2], (SyntaxKind::InterpStart, "${".to_string()));
    assert_eq!(tokens[3], (SyntaxKind::Ident, "name".to_string()));
    assert_eq!(tokens[4], (SyntaxKind::InterpEnd, "}".to_string()));
    assert_eq!(tokens[5], (SyntaxKind::DqStrEnd, "\"".to_string()));
  }

  #[test]
  fn yaml_mapping_line() {
    let tokens = lex_yaml("key: \"value\"");
    assert_eq!(tokens[0], (SyntaxKind::Ident, "key".to_string()));
    assert_eq!(tokens[1], (SyntaxKind::Colon, ":".to_string()));
    assert_eq!(tokens[2], (SyntaxKind::Whitespace, " ".to_string()));
    assert_eq!(tokens[3], (SyntaxKind::DqStrStart, "\"".to_string()));
    assert_eq!(tokens[4], (SyntaxKind::DqStrContent, "value".to_string()));
    assert_eq!(tokens[5], (SyntaxKind::DqStrEnd, "\"".to_string()));
  }

  /* Markdown mode tests */

  #[test]
  fn md_text() {
    let tokens = lex_markdown("hello");
    assert_eq!(tokens[0], (SyntaxKind::Ident, "hello".to_string()));
  }

  #[test]
  fn md_symbol_hash() {
    let tokens = lex_markdown("##");
    assert_eq!(tokens[0], (SyntaxKind::MdSymbol, "##".to_string()));
  }

  #[test]
  fn md_symbol_bold() {
    let tokens = lex_markdown("**");
    assert_eq!(tokens[0], (SyntaxKind::MdSymbol, "**".to_string()));
  }

  #[test]
  fn md_brackets() {
    let tokens = lex_markdown("[link](url)");
    assert_eq!(tokens[0], (SyntaxKind::LBracket, "[".to_string()));
    assert_eq!(tokens[1], (SyntaxKind::Ident, "link".to_string()));
    assert_eq!(tokens[2], (SyntaxKind::RBracket, "]".to_string()));
    assert_eq!(tokens[3], (SyntaxKind::LParen, "(".to_string()));
    assert_eq!(tokens[4], (SyntaxKind::Ident, "url".to_string()));
    assert_eq!(tokens[5], (SyntaxKind::RParen, ")".to_string()));
  }

  #[test]
  fn md_inline_math() {
    let tokens = lex_markdown("$E = mc^2$");
    assert_eq!(
      tokens[0],
      (SyntaxKind::InlineMath, "$E = mc^2$".to_string())
    );
  }

  #[test]
  fn md_formula() {
    let tokens = lex_markdown("${name}");
    assert_eq!(tokens[0], (SyntaxKind::InterpStart, "${".to_string()));
    assert_eq!(tokens[1], (SyntaxKind::Ident, "name".to_string()));
    assert_eq!(tokens[2], (SyntaxKind::InterpEnd, "}".to_string()));
  }

  #[test]
  fn md_inline_code() {
    let tokens = lex_markdown("`code`");
    assert_eq!(tokens[0], (SyntaxKind::InlineCode, "`code`".to_string()));
  }

  #[test]
  fn md_code_block() {
    let tokens = lex_markdown("```\ncode\n```");
    assert_eq!(
      tokens[0],
      (SyntaxKind::CodeBlock, "```\ncode\n```".to_string())
    );
  }

  #[test]
  fn md_newline() {
    let tokens = lex_markdown("\n");
    assert_eq!(tokens[0], (SyntaxKind::Newline, "\n".to_string()));
  }

  #[test]
  fn md_whitespace() {
    let tokens = lex_markdown(" ");
    assert_eq!(tokens[0], (SyntaxKind::Whitespace, " ".to_string()));
  }

  #[test]
  fn md_number() {
    let tokens = lex_markdown("42");
    assert_eq!(tokens.len(), 2); // Number + Eof
    assert_eq!(tokens[0], (SyntaxKind::Number, "42".to_string()));
  }

  #[test]
  fn md_math_block() {
    let tokens = lex_markdown("$$\nx + y\n$$");
    assert_eq!(tokens.len(), 2); // MathBlock + Eof
    assert_eq!(
      tokens[0],
      (SyntaxKind::MathBlock, "$$\nx + y\n$$".to_string())
    );
  }

  #[test]
  fn md_math_block_triple_dollar() {
    let tokens = lex_markdown("$$$\nx + y\n$$$");
    assert_eq!(tokens.len(), 2);
    assert_eq!(
      tokens[0],
      (SyntaxKind::MathBlock, "$$$\nx + y\n$$$".to_string())
    );
  }

  #[test]
  fn md_inline_math_double_dollar() {
    // $$ without newline is inline math
    let tokens = lex_markdown("$$x + y$$");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0], (SyntaxKind::InlineMath, "$$x + y$$".to_string()));
  }

  #[test]
  fn md_dollar_inside_double_dollar() {
    // $ inside $$ is literal
    let tokens = lex_markdown("$$x = $100$$");
    assert_eq!(tokens.len(), 2);
    assert_eq!(
      tokens[0],
      (SyntaxKind::InlineMath, "$$x = $100$$".to_string())
    );
  }

  #[test]
  fn md_unterminated_math() {
    let tokens = lex_markdown("$hello");
    assert_eq!(tokens[0].0, SyntaxKind::Error);
  }
}
