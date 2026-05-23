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
  InterpExpr,      // ${ ... }
  TaggedExpr,      // !tag value
  Tag,             // !string, !number, ...
  LiteralBlockStr, // | block scalar (preserves newlines)
  FoldedBlockStr,  // > block scalar (folds newlines to spaces)

  // Shared tokens
  Ident = 400,
  Number,
  DqStrStart,   // opening "
  DqStrContent, // text between " and " or ${
  DqStrEnd,     // closing "
  SqStrStart,   // opening '
  SqStrContent, // text between ' and ' or ${
  SqStrEnd,     // closing '
  InterpStart,  // ${
  InterpEnd,    // } closing an interpolation
  Colon,        // :
  Comma,        // ,
  LParen,       // (
  RParen,       // )
  LBracket,     // [
  RBracket,     // ]
  LBrace,       // {
  RBrace,       // }

  InlineMath, // matched $ delimiters, content on same line
  MathBlock,  // matched $ delimiters, content between newlines
  InlineCode, // matched ` delimiters, content on same line
  CodeBlock,  // matched ` delimiters, optional language tag, content between newlines

  // YAML mode tokens
  YamlOp,      // operators: +, -, ., ->, ==, !string, etc.
  YamlComment, // # ...
  YamlIndent,
  YamlDedent,

  // Markdown mode tokens
  MdSymbol, // any consecutive special chars (#, **, ~~, ---, :::, etc.)

  // Trivia
  Whitespace = 600,
  Newline,
  Eof,

  // Error
  Error,
}
