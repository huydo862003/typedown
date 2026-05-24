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
  PrimaryExpr, // An operand in an expression
  ParenExpr,   // (expr)
  UnaryExpr,
  BinaryExpr,

  // Literals
  // All literals must be wrapped in a primary expr to be treated as an expression
  TaggedLit,          // !tag value
  ListLit,            // Flow sequence in yaml frontmatter & formula mode
  BlockSeqLit,        // Block sequence in yaml frontmatter
  MappingLit,         // Flow mapping in yaml frontmatter & formula mode
  LiteralBlockStrLit, // | block scalar (preserves newlines)
  FoldedBlockStrLit,  // > block scalar (folds newlines to spaces)
  BlockMappingLit,    // Block mapping in yaml frontmatter
  StrLit,             // String literal + interpolation + math
  InterpFragment,     // Interpolation fragment: ${...}
  MathLit,            // Inline + block math expression
  CodeLit,            // Inline + block code expression
  NumberLit,
  IdentLit,

  Tag, // !string, !number, ...
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
