# Typedown

![Status](https://img.shields.io/badge/status-active-brightblue)
![License](https://img.shields.io/badge/license-GPL-green)
<a href="https://github.com/huydo862003/Fck-AI-Slop#plan"><img src="https://img.shields.io/badge/Human%20slop-90EE90"></a>

A typed markdown language for structured content.

## Neovim Plugin

The Neovim LSP client lives in `editors/nvim/`. To test it without affecting your regular Neovim config:

```bash
nvim -u editors/nvim/test_init.lua
```

Then run `:LspInfo` inside Neovim to verify the server attached. Make sure `typedown-lsp` is built first:

```bash
cargo build --release
```

## Common Pitfalls (and Painful Lessons)

These are some lessons learnt during the development of the project. Some comments in the code are also marked with `TIL`.

### Visitor Pattern for Serialization/Hashing

There are two naive approaches to serialization, and a third that combines the best of both. This applies to why the built-in Hash trait chooses this design.

**Approach 1: Serializer knows every type**: The serializer has a method per type. Adding a new type means modifying the serializer.

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

**Approach 2: Each type serializes itself**: Each type writes its own bytes. But now every type must know the byte format, and changing the format means updating every type.

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

**Approach 3: Visitor (double dispatch)**. Split the responsibilities. The type decides WHAT to write (which fields, in what order). The serializer decides HOW to write it (byte format, endianness, buffering). Neither depends on the other's internals.

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
