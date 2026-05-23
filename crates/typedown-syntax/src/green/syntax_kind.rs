#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
  // Top-level
  SourceFile = 0, // root: frontmatter + body

  // Frontmatter (YAML mode) nodes
  Frontmatter = 100,
  Mapping,
  MappingEntry, // one key: value pair
  Key,
  Value,
  Sequence,
  SequenceItem, // one - item

  // Body (Markdown mode) nodes
  Body = 200,
  Heading,
  Paragraph,
  Blockquote,
  Table,
  TableRow,
  TableCell,
  BulletList,
  BulletListItem,
  OrderedList,
  OrderedListItem,
  ToggleList,
  ToggleListItem,
  CalloutBlock,      // ::: label ... :::
  FootnoteBlock,     // ::: footnote ... :::
  BibliographyBlock, // ::: bibtex ... :::
  Link,              // [text](url)
  Media,             // ![alt](src)
  FootnoteRef,       // [^key]
  Citation,          // [@key]
  Text,              // plain text run

  // Expression nodes
  Expr = 300,
  InterpExpr, // ${ ... }
  TaggedExpr, // !tag value
  Tag,        // !string, !number, ...

  // Tokens (leaf nodes emitted by the lexer)
  Ident = 400,
  Number,
  DqStr,      // "..."
  SqStr,      // '...'
  InlineCode, // matched ` delimiters, content on same line
  CodeBlock,  // matched ` delimiters, optional language tag, content between newlines
  InlineMath, // matched $ delimiters, content on same line
  MathBlock,  // matched $ delimiters, content between newlines
  YamlOp,     // operators: +, -, ., ->, ==, etc.

  // Punctuation and delimiters
  TripleDash = 500, // ---
  TripleColon,      // :::
  Colon,            // :
  Bang,             // !
  Dollar,           // $
  LParen,           // (
  RParen,           // )
  LBracket,         // [
  RBracket,         // ]
  LBrace,           // {
  RBrace,           // }
  Comma,            // ,
  Pipe,             // |
  Hash,             // #
  At,               // @
  Caret,            // ^
  Star,             // *
  Tilde,            // ~
  InterpStart,      // ${
  InterpEnd,        // } closing an interpolation

  // Trivia
  Whitespace = 600,
  Newline,
  Comment, // # ...
  Indent,
  Dedent,
  Eof,

  // Error
  Error,
}
