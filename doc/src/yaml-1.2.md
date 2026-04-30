# YAML 1.2

Link: https://yaml.org/spec/1.2.2/

## Overview

YAML 1.2 has evolved to:

- Be a strict superset of JSON.
- Remove many obscure rules like implicit typings.

The general important [goals](https://yaml.org/spec/1.2.2/#goals) of YAML:

1. Be human-readable.
2. Be portable across languages.
3. Easy to parse/implement/use.

Context: PyYAML is generally considered the reference information. LibYAML is generally used in other YAML frameworks.

## Structure

To understand YAML, one should:

- Understand its information model, that is, the abstract model of how YAML decides to organize data.
- Translation of that information model to the YAML textual format.

## Features Overview

This section lists all features of YAML, to motivate the YAML specification later.

Three basic building blocks: Scalars, Sequences, Mappings. (for more details check [Scalars](#scalars) onwards)

### Syntax

#### Block Syntax

- Indentation is used for scope: Same indentation = Same list/hash.
- A line is used for specifying an entry within the list/hash.

Entry syntax:

- **Block sequences** use a dash and a space (`- `) for each entry.
- **Block mappings** use a colon and a space (`: `) for each key/value pair.

Comments use `#`.

Example:

```yaml
house_1:
  - person_1 # First entry of the first key/value pair
  - hrnax # Second entry of the first key/value pair
```

Example from the spec:

```yaml
- name: Mark McGwire
  hr: 65
  avg: 0.278
- name: Sammy Sosa
  hr: 63
  avg: 0.288
```

For a more compact representation, see [Compact Nested Mapping](#compact-nested-mapping).

#### Flow Syntax

- Indicators are used to denote scopes.
- Puntuations are used to denote entries.

Entry syntax:

- **Flow sequences** use a comma-separated list within square brackets.
- **Flow mappings** use a comma-separated list of key/value pairs within curly braces.

```yaml
flow mapping: { a: 1, b: 2 }

flow sequence:
  - [1, 2, 3]
  - [4, 5, 6]
```

#### Data Sharing

In data structures, it's common to have an object/list to be shared in more than one places.

YAML allows this by **anchors** (`&`) and **aliases** (`*`), kind of like C++ address-of and dereferencing operators.

Example from the spec:

```yaml
hr:
  - Mark McGwire
  # Following node labeled SS
  - &SS Sammy Sosa
rbi:
  - *SS # Subsequent occurrence
  - Ken Griffey
```

#### Complex Mapping Keys

Sometimes mapping keys need to be more than simple scalars, e.g using a list as a key.

A question mark and space (`? `) indicates a complex mapping key. Within a block collection, key/value pairs can start immediately following the dash, colon or question mark.

Example from the spec:

```yaml
? - Detroit Tigers
  - Chicago cubs
: - 2001-07-23

[New York Yankees, Atlanta Braves]: [2001-07-02, 2001-08-12, 2001-08-14]
```

#### Compact Nested Mapping

It's common to have a list of objects (e.g. a product catalog).

Within a block sequence, key/value pairs can start immediately after the dash, allowing compact representation of lists of mappings.

Example from the spec:

```yaml
# Products purchased
- item: Super Hoop
  quantity: 1
- item: Basketball
  quantity: 4
- item: Big Shoes
  quantity: 1
```

### Structures

YAML files can be made up of **directives** and **document content**.

- **Directives**: instructions for the YAML processor, not part of the data. YAML 1.2 defines 2 directives: `%YAML` (version) and `%TAG` (tag shorthands).
- **Document content**: the actual data described in the above section, such as scalars, sequences, and mappings.

Syntactically:

- `---` is used to separate directives from content, and signals start of a new document (even without directives, effectively just document separators).

  Example from the spec:

  ```yaml
  # Ranking of 1998 home runs
  ---
  - Mark McGwire
  - Sammy Sosa
  - Ken Griffey

  # Team ranking
  ---
  - Chicago Cubs
  - St Louis Cardinals
  ```

- `...` is used to signal the end of a document without starting a new one.

  Example from the spec:

  ```yaml
  ---
  time: 20:03:20
  player: Sammy Sosa
  action: strike (miss)
  ...
  ---
  time: 20:03:47
  player: Sammy Sosa
  action: grand slam
  ...
  ```

### Scalars

Scalars are YAML's atomic values: strings, numbers, booleans, etc. Unlike sequences and mappings, they hold a single value.

Scalar content can be written in two notations: **block** and **flow**.

#### Block Scalars

Block scalars are useful for multi-line strings where whitespace and newline control matters (e.g. embedded config, prose, ASCII art).

In block scalars, the base indentation is determined by the first non-empty content line. That indentation is stripped from all lines.

Block scalars have two styles:

- **Literal style** (`|`): all line breaks are preserved as-is.
- **Folded style** (`>`): newlines are folded to spaces, **except** lines that are blank or more-indented than the base, those preserve their newlines.

Examples:

- In literal style (`|`), newlines are preserved.

  Example:

  ```yaml
  newlines_preserved: |
    First line
    Second line
    Third line
  ```

  Interpreted: `"First line\nSecond line\nThird line\n"`

- In folded style (`>`), newlines become spaces. Newlines are preserved for more-indented and blank lines.

  Example from the spec:

  ```yaml
  description: >
    Mark McGwire's
    year was crippled
    by a knee injury.
  ```

  Interpreted: `"Mark McGwire's year was crippled by a knee injury.\n"`

  Example from the spec (preserved newlines for more-indented and blank lines):

  ```yaml
  summary: >
    Sammy Sosa completed another
    fine season with great stats.

      63 Home Runs
      0.288 Batting Average

    What a year!
  ```

  Interpreted: `"Sammy Sosa completed another fine season with great stats.\n\n  63 Home Runs\n  0.288 Batting Average\n\nWhat a year!\n"`

- A document's root node can itself be a scalar, the block scalar indicator follows `---` directly ([§9.1.3. Bare Documents](https://yaml.org/spec/1.2.2/#913-bare-documents)):

  ```yaml
  --- |
    entire document
    is one literal scalar
  ```

- Indentation determines scope ([§6.1. Indentation Spaces](https://yaml.org/spec/1.2.2/#61-indentation-spaces)): a block scalar ends when indentation drops back to the parent's level. Lines below the base indentation but above the parent's level are invalid ([§8.1.1. Block Scalar Headers](https://yaml.org/spec/1.2.2/#811-block-scalar-headers)):

  Example from the spec:

  ```yaml
  name: Mark McGwire
  accomplishment: >
    Mark set a major league
    home run record in 1998.
  stats: |
    65 Home Runs
    0.278 Batting Average
  ```

#### Flow Scalars

Flow scalars are inline, useful for short strings, values with special characters, or when you want to stay on one line.

YAML's flow scalars have 3 styles:

- The **plain style**
- Two **quoted styles**:
  - The double-quoted style provides escape sequences.
  - The single-quoted style is useful when escaping is not needed.

All flow scalars can span multiple lines. Line breaks are always folded.

- Quoted scalars:

  Example from the spec:

  ```yaml
  unicode: "Sosa did fine.\u263A"
  control: "\b1998\t1999\t2000\n"
  hex esc: "\x0d\x0a is \r\n"

  single: '"Howdy!" he cried.'
  quoted: " # Not a 'comment'."
  tie-fighter: '|\-*-/|'
  ```

- Multi-line flow scalars:

  Example from the spec:

  ```yaml
  plain: This unquoted scalar
    spans many lines.

  quoted: "So does this
    quoted scalar.\n"
  ```

### Tags

Every YAML node has a **tag** that denotes its type (e.g `!!str`, `!!int`, `!!seq`).

For simplicity though, most nodes don't specify one explicitly: they are **untagged** and the YAML processor resolves a tag automatically based on the active **schema**, which defines the set of available tags and how untagged nodes are resolved (e.g `123` -> `!!int`, `true` -> `!!bool`).

YAML 1.2 defines three built-in schemas:

- **Fail safe schema**: the minimum every processor must support. Only `!!str`, `!!seq`, `!!map`.
- **JSON schema** (recommended): adds `!!null`, `!!bool`, `!!int`, `!!float`.
- **Core schema** (recommended): extends JSON schema with human-friendly notations (e.g octal `0o14`, hex `0xC`, `~` for null).

The examples in this specification generally use the `seq`, `map` and `str` types from the fail safe schema. A few examples also use the `int`, `float` and `null` types from the JSON schema.

- Integers:

  Example from the spec:

  ```yaml
  canonical: 12345
  decimal: +12345
  octal: 0o14
  hexadecimal: 0xC
  ```

- Floating point:

  Example from the spec:

  ```yaml
  canonical: 1.23015e+3
  exponential: 12.3015e+02
  fixed: 1230.15
  negative infinity: -.inf
  not a number: .nan
  ```

- Miscellaneous:

  Example from the spec:

  ```yaml
  null:
  booleans: [true, false]
  string: "012345"
  ```

- Timestamps:

  Example from the spec:

  ```yaml
  canonical: 2001-12-15T02:59:43.1Z
  iso8601: 2001-12-14t21:59:43.10-05:00
  spaced: 2001-12-14 21:59:43.10 -5
  date: 2002-12-14
  ```

#### Explicit Tags

Explicit typing is denoted with a tag using the exclamation point (`!`) symbol.

There are two kinds of tags:

- **Local tags**:
  - Motivation: Sometimes tags are only required to have meanings within the consuming application.
    - Start with `!` and are application-specific (e.g `!circle`).
    - Don't need to be declared, you just use them inline.
- **Global tags**:
  - Motivation: When interoperability matters, tags that are universally recognized across different processors are useful. 
  - Syntax: full URIs that are universally unique (e.g `tag:yaml.org,2002:str`), standardized across all processors.

Since global tags are verbose, YAML provides **tag handles** (also called tag shorthands): short prefixes that expand to a URI prefix, declared via the `%TAG` directive.

Two handles are built-in:

- `!!` defaults to `tag:yaml.org,2002:`, so `!!str` expands to `tag:yaml.org,2002:str`.
- `!` is the primary handle for local tags.

Custom handles can also be defined:

```yaml
%TAG !e! tag:example.com,2002:
---
- !e!circle   # expands to tag:example.com,2002:circle
  radius: 7
```

- Example from the spec:

  ```yaml
  %TAG ! tag:clarkevans.com,2002:
  ---
  !shape
  # Use the ! handle for presenting
  # tag:clarkevans.com,2002:circle
  - !circle
    center: &ORIGIN { x: 73, y: 129 }
    radius: 7
  - !line
    start: *ORIGIN
    finish: { x: 89, y: 102 }
  - !label
    start: *ORIGIN
    color: 0xFFEEBB
    text: Pretty vector drawing.
  ```

- Unordered sets:

  Example from the spec:

  ```yaml
  # Sets are represented as a
  # Mapping where each key is
  # associated with a null value
  --- !!set
  ? Mark McGwire
  ? Sammy Sosa
  ? Ken Griffey
  ```

- Ordered mappings:

  Example from the spec:

  ```yaml
  # Ordered maps are represented as
  # A sequence of mappings, with
  # each mapping having one key
  --- !!omap
  - Mark McGwire: 65
  - Sammy Sosa: 63
  - Ken Griffey: 58
  ```
