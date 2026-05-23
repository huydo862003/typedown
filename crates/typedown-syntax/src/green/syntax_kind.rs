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

  // Shared tokens
  Ident = 400,
  Number,
  DqStrStart,   // opening "
  DqStrContent, // text between " and " or ${
  DqStrEnd,     // closing "
  SqStrStart,   // opening '
  SqStrContent, // text between ' and ' or ${
  SqStrEnd,     // closing '

  InterpStart = 480, // ${
  InterpEnd,         // } closing an interpolation

  // YAML mode tokens
  YamlOp = 420, // operators: +, -, ., ->, ==, !string, etc.
  YamlColon,    // :
  YamlComma,    // ,
  YamlLParen,   // (
  YamlRParen,   // )
  YamlLBracket, // [
  YamlRBracket, // ]
  YamlLBrace,   // {
  YamlRBrace,   // }
  YamlComment,  // # ...
  YamlIndent,
  YamlDedent,

  // Markdown mode tokens
  MdSymbol = 450, // any consecutive special chars (#, **, ~~, ---, :::, etc.)
  MdLBracket,     // [
  MdRBracket,     // ]
  MdLParen,       // (
  MdRParen,       // )
  MdDollar,       // $
  MdInlineCode,   // matched ` delimiters, content on same line
  MdCodeBlock,    // matched ` delimiters, optional language tag, content between newlines
  MdInlineMath,   // matched $ delimiters, content on same line
  MdMathBlock,    // matched $ delimiters, content between newlines

  // Trivia
  Whitespace = 600,
  Newline,
  Eof,

  // Error
  Error,
}
