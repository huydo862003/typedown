#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
  // Top-level
  SourceFile = 0, // root: frontmatter + body

  // Frontmatter (YAML mode) nodes
  Frontmatter = 100,
  Mapping,           // block mapping (indentation-based)
  MappingEntry,      // one key: value pair in block mapping
  MappingEntryKey,   // key in a block mapping entry
  MappingEntryValue, // value in a block mapping entry
  Sequence,
  SequenceItem, // one - item

  // Body (Markdown mode) nodes
  Body = 200,
  Heading,
  Paragraph,
  Blockquote,
  Table,
  TableRow,
  TableSeparatorRow,
  TableCell,
  BulletList,
  BulletListItem,
  OrderedList,
  OrderedListItem,
  ToggleList,
  ToggleListItem,
  ToggleListSummary,
  ToggleListDetails,
  CalloutBlock,      // ::: label ... :::
  Link,              // [text](url)
  Media,             // ![alt](src)
  FootnoteRef,       // [^key]
  Citation,          // [@key]
  Bold,              // **text**
  Italic,            // *text* or _text_
  BoldItalic,        // ***text***
  Strikethrough,     // ~~text~~
  Text,              // plain text run

  // Expression nodes
  Expr = 300,
  PrimaryExpr, // An operand in an expression
  ParenExpr,   // (expr)
  CallExpr,    // func(args)
  UnaryExpr,
  BinaryExpr,

  // Literals
  // All literals must be wrapped in a primary expr to be treated as an expression
  TaggedLit,          // !tag value
  ListLit,            // Flow sequence in yaml frontmatter & formula mode
  BlockSeqLit,        // Block sequence in yaml frontmatter
  DictLit,            // Flow mapping `{key: value, ...}` in yaml frontmatter & formula mode
  DictEntry,          // one key: value pair in a dict
  DictEntryKey,       // key in a dict entry
  DictEntryValue,     // value in a dict entry
  LiteralBlockStrLit, // | block scalar (preserves newlines)
  FoldedBlockStrLit,  // > block scalar (folds newlines to spaces)
  BlockMappingLit,    // Block mapping in yaml frontmatter (indentation-based)
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
  MdSymbol,    // any consecutive special chars (#, **, ~~, ---, :::, etc.)
  MdNumber,    // integer only (for ordered list markers like `1.`, `23.`)
  HtmlEntity,  // &name; or &#digits; or &#xhex; (e.g. &amp;, &#42;, &#x2A;)

  // Trivia
  Whitespace = 600,
  Newline,
  Eof,

  // Error
  Error,
}
