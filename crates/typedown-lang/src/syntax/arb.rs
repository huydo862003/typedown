//! Arbitrary implementations for syntax types, used in property-based tests.

use proptest::prelude::*;
use strum::IntoEnumIterator;

use crate::syntax::green::GreenNode;
use crate::syntax::green::cache::Cache;
use crate::syntax::green::token::SyntaxToken;
use crate::syntax::red::RedNode;
use crate::syntax::syntax_kind::SyntaxKind;

impl Arbitrary for SyntaxKind {
  type Parameters = ();
  type Strategy = BoxedStrategy<Self>;

  fn arbitrary_with(_: ()) -> Self::Strategy {
    let all: Vec<SyntaxKind> = SyntaxKind::iter().collect();
    prop::sample::select(all).boxed()
  }
}

impl Arbitrary for GreenNode {
  type Parameters = ();
  type Strategy = BoxedStrategy<Self>;

  fn arbitrary_with(_: ()) -> Self::Strategy {
    arb_green_token()
      .prop_recursive(4, 64, 8, |inner| arb_green_node(inner))
      .boxed()
  }
}

impl Arbitrary for RedNode {
  type Parameters = ();
  type Strategy = BoxedStrategy<Self>;

  fn arbitrary_with(_: ()) -> Self::Strategy {
    // RedNode root must be a node, not a token
    arb_green_node(arb_green_token().prop_recursive(3, 64, 8, |inner| arb_green_node(inner)))
      .prop_map(|green| RedNode::from_green(0, green))
      .boxed()
  }
}

fn arb_green_token() -> impl Strategy<Value = GreenNode> {
  (
    any::<SyntaxKind>(),
    proptest::collection::vec(any::<char>(), 0..32),
  )
    .prop_map(|(kind, bytes)| {
      let token =
        SyntaxToken::from_raw_parts(kind, bytes.into_iter().collect::<String>().into_bytes());
      GreenNode::from_token(token)
    })
}

fn arb_green_node(
  child: impl Strategy<Value = GreenNode> + 'static,
) -> impl Strategy<Value = GreenNode> {
  (any::<SyntaxKind>(), proptest::collection::vec(child, 0..8)).prop_map(|(kind, children)| {
    let mut cache = Cache::new();
    let node = cache.node(kind, &children);
    GreenNode::from_node(node)
  })
}
