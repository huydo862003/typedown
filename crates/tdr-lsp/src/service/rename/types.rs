use tdr_lang::syntax::ast::{IdentLit, StrLit};

pub enum RenameSymbol {
  Fref { string_node: StrLit },
  Identifier { node: IdentLit },
}
