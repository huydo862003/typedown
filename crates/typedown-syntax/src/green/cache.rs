//! A non-thread safe interner for node/token deduplication.
//! Cannot ensure across-thread uniqueness.
//! Equal pointers -> Equal node/token.
//! Different pointers -> Not sure.

use hashbrown::HashMap;
use hashbrown::hash_map::RawEntryMut;
use std::hash::{BuildHasher, Hash, Hasher};

use super::child::GreenChild;
use super::node::Node;
use super::syntax_kind::SyntaxKind;
use super::token::Token;

/// A non-thread safe interner for node/token deduplication.
#[derive(Default)]
pub struct Cache {
  // We use HashMap instead of HashSet to access the raw entry API,
  // which avoids allocating just to check if an entry exists.
  tokens: HashMap<Token, ()>,
  nodes: HashMap<Node, ()>,
}

impl Cache {
  pub fn new() -> Self {
    Self::default()
  }

  /// Get or create an interned token.
  pub fn token(&mut self, kind: SyntaxKind, bytes: &[u8]) -> Token {
    // Hash from borrowed data to avoid allocating a String on cache hit
    let hash = {
      let mut h = self.tokens.hasher().build_hasher();
      kind.hash(&mut h);
      bytes.hash(&mut h);
      h.finish()
    };

    let entry = self.tokens.raw_entry_mut().from_hash(hash, |existing| {
      existing.kind() == kind && existing.bytes() == bytes
    });

    match entry {
      RawEntryMut::Occupied(e) => e.key().clone(),
      RawEntryMut::Vacant(e) => {
        let token = Token::from_raw_parts(kind, bytes.to_vec());
        e.insert_hashed_nocheck(hash, token.clone(), ());
        token
      }
    }
  }

  /// Get or create an interned node.
  pub fn node(&mut self, kind: SyntaxKind, children: &[GreenChild]) -> Node {
    let hash = {
      let mut h = self.nodes.hasher().build_hasher();
      kind.hash(&mut h);
      children.hash(&mut h);
      h.finish()
    };

    let entry = self.nodes.raw_entry_mut().from_hash(hash, |existing| {
      existing.kind() == kind && existing.children() == children
    });

    match entry {
      RawEntryMut::Occupied(e) => e.key().clone(),
      RawEntryMut::Vacant(e) => {
        let node = Node::from_raw_parts(kind, children);
        e.insert_hashed_nocheck(hash, node.clone(), ());
        node
      }
    }
  }
}
