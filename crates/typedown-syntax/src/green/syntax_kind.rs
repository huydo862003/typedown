#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
  // Top-level
  SourceFile = 0, // root: frontmatter + body

  // Frontmatter
  Frontmatter = 100,

  Mapping,
  MappingEntry, // one key: value pair
  Key,
  Value,

  Sequence,
  SequenceItem, // one - item

  // Body (TDR Markdown)
  Body = 200,
  Heading,
  Paragraph,
  PlainText, // Optimized for long proses for memort efficiency
  Blockquote,
  CodeBlock,
  InlineCode,
  MathBlock,
  InlineMath,
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
  FootnoteBlock,     // ::: footnote
  BibliographyBlock, // ::: bibtex
  Link,              // [text](url)
  Media,             // ![alt](src)
  FootnoteRef,       // [^key]
  Citation,          // [@key]
  Text,              // plain text run

  // Expressions
  Expr = 300,
  InterpExpr, // ${ ... }
  TaggedExpr, // !tag value
  Tag,        // !string, !number, ...

  // Tokens
  Ident = 400,
  Number,
  DqStr, // "..."
  SqStr, // '...'
  BtStr, // `...`

  // Punctuation / markers
  TripleDash = 500, // ---
  TripleColon,      // :::
  Colon,            // :
  Bang,             // !
  Dollar,           // $
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
  Backtick,         // `
  InterpStart,      // ${
  InterpEnd,        // $}

  // Trivia
  Whitespace = 600,
  Newline,
  Comment,

  // Error
  Error,
}
