# Typedown

![Status](https://img.shields.io/badge/status-active-brightblue)
![License](https://img.shields.io/badge/license-GPL-green)
<a href="https://github.com/huydo862003/Fck-AI-Slop#plan"><img src="https://img.shields.io/badge/Human%20slop-90EE90"></a>

A typed markdown language for structured content.

## Design Documentation

The compiler design is researched and documented in the [dboxide](https://github.com/Huy-DNA/dboxide) repo. See the [design docs](https://github.com/Huy-DNA/dboxide/tree/main/doc/src/design) for details on the syntax, type system, and incremental compilation engine.

The tree-sitter grammar research is documented in the [loupe](https://github.com/huydo862003/loupe) repo.

## Dev Setup

See [DEVELOPMENT.md](DEVELOPMENT.md) for full setup instructions (Nix and non-Nix).

## Editor Integration

| Editor            | User Guide                         | Development                                  |
| ----------------- | ---------------------------------- | -------------------------------------------- |
| Neovim            | [README](editors/nvim/README.md)   | [DEVELOPMENT](editors/nvim/DEVELOPMENT.md)   |
| Zed               | [README](editors/zed/README.md)    | [DEVELOPMENT](editors/zed/DEVELOPMENT.md)    |
| VSCode / VSCodium | [README](editors/vscode/README.md) | [DEVELOPMENT](editors/vscode/DEVELOPMENT.md) |

## Dependency Graph

- `tdr-macros` and `tdr-types` contain common utils, which are the lowest common denominator that everyone depends upon.
  - They can be depended upon by other crates.
  - They must not depend on any other crates.
- `tdr-incremental` contains the incremental engine.
  - It must not depend on any other crates, except for `tdr-macros` and `tdr-types`.
  - It can be depended upon by everyone, EXCEPT FOR `tdr-macros` and `tdr-types`.
- `tdr-lang` contains the AST structure, parser, typechecking, and evaluation logic for typedown.
  - It depends on `tdr-incremental`, `tdr-macros`, and `tdr-types`.
  - It must not depend on `tdr-lsp`.
  - It can only be depended upon by `tdr-lsp`.
- `tdr-lsp` contains the LSP server for typedown.
  - It can depend on any other crates.
  - It can not be depended upon by others.

## Common Pitfalls (and Painful Lessons)

These are some lessons learnt during the development of the project. Some comments in the code are also marked with `TIL`.

### Visitor Pattern for Serialization/Hashing

There are two naive approaches to serialization, and a third that combines the best of both. This applies to why the built-in Hash trait chooses this design.

> I think this is related to the expression problem.

**Approach 1: Serializer knows every type**:

- There's a single serializer.
- The serializer has a method per type to serialize objects of that type.
- Adding a new type means modifying the serializer.

```rust
struct Serializer { buf: Vec<u8> }

impl Serializer {
    fn serialize_person(&mut self, p: &Person) {
        self.buf.extend(p.name.as_bytes());
        self.buf.extend(&p.age.to_le_bytes());
    }
    fn serialize_product(&mut self, p: &Product) { /* ... */ }
    // Every new type = new method here
}
```

**Approach 2: Each type serializes itself**:

- Each type handles its own serialization.
- Now every type must know the byte format, and changing the format means updating every type.

```rust
trait Serialize {
    fn serialize(&self, buf: &mut Vec<u8>);
}

impl Serialize for Person {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.extend(self.name.as_bytes());  // must know the wire format
        buf.extend(&self.age.to_le_bytes());
    }
}
```

**Approach 3: Visitor (double dispatch)**. Split the responsibilities.

- The type decides WHAT to write (which fields, in what order).
- The serializer decides HOW to write it (byte format, endianness, buffering).
- Therefore, neither depends on the other's internals.

```rust
trait Serializer {
    fn emit_str(&mut self, v: &str);
    fn emit_u32(&mut self, v: u32);
}

trait Serialize {
    fn serialize(&self, s: &mut impl Serializer);
}

impl Serialize for Person {
    fn serialize(&self, s: &mut impl Serializer) {
        s.emit_str(&self.name);  // WHAT: name field
        s.emit_u32(self.age);    // WHAT: age field
    }
}
```

Adding a new type does not touch the serializer. Changing the byte format does not touch any type. This is how `std::hash` works (`Hash`/`Hasher`), how rustc does it (`Encodable`/`Encoder`), and how serde does it (`Serialize`/`Serializer`).

Reference: [rustc_serialize/src/serialize.rs](https://github.com/rust-lang/rust/blob/2371d697abddba53be85137d5a68064066b4ae10/compiler/rustc_serialize/src/serialize.rs)

### LSP: Dynamic vs Static Registration

When a client advertises `dynamicRegistration: true` for `workspace.fileOperations` (as VSCode does), some clients **ignore** static capabilities declared in `InitializeResult`. The server must use `client/registerCapability` to dynamically register for `workspace/willRenameFiles` and `workspace/didRenameFiles` at runtime.
