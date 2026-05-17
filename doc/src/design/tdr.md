# TDR: Typedown Resource

TDR (Typedown Resource) is the serialization format for Typedown resources.

- [Basic Structure](#basic-structure)
- [TDR Resource](#tdr-resource)
  - [Comments](#comments)
  - [Top-level Frontmatter Value](#top-level-frontmatter-value)
  - [Type Declaration](#type-declaration)
  - [Label](#label)
  - [Properties](#properties)
  - [Links](#links)
- [TDR Expression](#tdr-expression)
  - [Scalars](#scalars)
  - [Lists](#lists)
  - [Records](#records)
  - [Type Expressions](#type-expressions)
- [TDR Explicit Type Tags](#tdr-explicit-type-tags)
- [TDR Schema](#tdr-schema)
- [TDR Markdown](#tdr-markdown)
  - [Headings](#headings)
  - [Code](#code)
  - [Blockquotes](#blockquotes)
  - [Math](#math)
  - [Tables](#tables)
  - [Lists](#lists)
  - [Callout Blocks](#callout-blocks)
  - [Multimedia](#multimedia)
  - [Links](#links-1)
  - [Footnotes](#footnotes)
  - [Bibliography](#bibliography)
  - [String Interpolation](#string-interpolation)

## Basic Structure

A `.tdr` file consists of two sections:

1. A YAML-like frontmatter block containing the resource's structured data (the **TDR frontmatter**, or **frontmatter**), followed by
2. A [TDR Markdown](#tdr-markdown) body for free-form content (the **TDR body**, or **body**).

```
---
<frontmatter>
---

<body>
```

- The opening `---` is the frontmatter start marker.
- The closing `---` is the frontmatter end marker.
- Everything after belongs to the body.

The syntaxes will be familiar to anyone who has worked with YAML and Markdown. TDR is case-sensitive throughout: identifiers, property names, type names, and reserved keys like `$type` and `$label` must match exactly.

## TDR Resource

Every `.tdr` file is a **TDR resource file**. It contains a frontmatter and a body. The body is free-form content written in [TDR Markdown](#tdr-markdown). The frontmatter is where the resource's structured data lives.

### Comments

The frontmatter supports YAML line comments using `#`:

```yaml
---
first_name: Bob # this is a comment
---
```

### Top-level Frontmatter Value

The top-level frontmatter must be a **YAML mapping** with scalar string keys. A YAML mapping is a set of key-value pairs, similar to a JSON object. Keys must be simple strings with no special YAML tags or complex types.

The following are invalid as keys:

```yaml
[a, b]: value # sequence as key
{ a: 1 }: value # mapping as key
!type string: value # tagged key
&anchor key: value # anchored key
```

Valid keys are plain strings, optionally quoted:

```yaml
first_name: Bob
"birth_date": 1990-07-04
author: !link bob.tdr
```

### Type Declaration

A resource file must declare its type using `$type`. The value is the name of a [TDR Schema](#tdr-schema) that the resource conforms to. The schema enforces what properties the resource is expected to have.

For example, given a `person` schema defined as:

```yaml
---
$type: Schema
properties:
  first_name:
    type: !type string
    required: true
  last_name:
    type: !type string
    required: true
  birth_date:
    type: !type date
---
```

A resource conforming to it declares `$type: person` and must provide the required fields:

```yaml
---
$type: person
first_name: Bob
last_name: Smith
birth_date: 1990-07-04
---
```

Property values do not need explicit type tags when the type can be inferred from the schema. `Bob` above is inferred as a `string` because the schema declares `first_name` as `!type string`. Explicit tags like `!string Bob` are only needed when the type cannot be inferred.

A resource can also declare additional fields not defined in its schema. These are stored as-is and are not validated by the schema.

### Label

A resource file can declare a human-readable label using `$label`. The label is a [TDR Expression](#tdr-expression) and can reference other properties:

```yaml
---
$type: person
$label: !string self.first_name + " " + self.last_name
---
```

### Properties

All frontmatter keys other than reserved `$` keys are properties of the resource. Property values are [TDR Expressions](#tdr-expression).

```yaml
---
$type: person
$label: !string self.first_name + " " + self.last_name
first_name: Bob
birth_date: 1990-07-04
author: !link mona_lisa.tdr
tags:
  - research
  - rdf
---
Free-form markdown body content.
```

### Links

A link is a property tagged with `!link`, pointing to another `.tdr` file by filename. Links form directed edges in the resource graph.

```yaml
author: !link bob.tdr
```

A link can also reference a property that resolves to the target:

```yaml
author: !link self.default_author
```

Multi-valued links are expressed as a YAML sequence:

```yaml
tags:
  - !link research.tdr
  - !link rdf.tdr
```

## TDR Expression

Every value in TDR frontmatter is an expression. Each expression has a type. In most cases the type is inferred from the schema, so it does not need to be stated explicitly.

Whether a value is an identifier or a literal is inferred from context in most cases. In ambiguous contexts, identifiers are preferred. To force a literal interpretation, wrap the value in single or double quotes (e.g. `'draft'`, `"published"`). Identifiers that contain special characters are wrapped in backticks.

### Scalars

A scalar is a single primitive value. The scalar types are: `string`, `number`, `boolean`, `date`, `link`. The type is inferred from the schema:

```yaml
first_name: Bob # string
birth_date: 1990-07-04 # date
count: 42 # number
active: true # boolean
author: bob.tdr # link
```

String values support interpolation with `${}`. Any expression can appear inside the braces:

```yaml
greeting: "Hello, ${self.first_name}!"
summary: "Written by ${self.author.first_name} on ${self.created_at}"
```

### Lists

A list is a YAML sequence. Its type is `list[T]`, where `T` is the element type. Each element is itself an expression:

```yaml
tags: # list[string]
  - research
  - rdf
authors: # list[link]
  - !link bob.tdr
  - !link alice.tdr
```

### Records

A record is a YAML mapping nested under a property key. Each value is itself an expression. Records come in two forms:

`record[K, V]` is a homogeneous mapping where all keys share the same key type `K` and all values share the same value type `V`:

```yaml
scores: # record[string, number]
  alice: 95
  bob: 87
```

A fixed-key mapping assigns a specific type to each named key independently:

```yaml
address: # { street: string, city: string, zip: number }
  street: Main St
  city: Springfield
  zip: 12345
```

### Type Expressions

A type expression resolves to a type value rather than a data value. Type expressions use the `!type` tag and are only valid in schema property definitions. The built-in types are: `string`, `number`, `boolean`, `date`, `enum`, `list[T]`, `record[K, V]`, `link[schema]`, and literal types:

```yaml
type: !type string
type: !type list[string]
type: !type record[string, number]
type: !type link[person]
```

A fixed-key record type is expressed as a YAML mapping under the `!type` tag, either flow or block:

```yaml
type: !type { street: string, city: string, zip: number }  # flow
type: !type           # block
  street: string
  city: string
  zip: number
```

An enum type is expressed as a YAML sequence of literal values under the `!type` tag. Each element is a literal of any type. String literals must be quoted to distinguish them from type name identifiers; number and boolean literals are unambiguous without quotes:

```yaml
type: !type ['draft', 'published', 'archived']  # string enum, flow
type: !type [1, 2, 3]                           # number enum, flow
type: !type                                     # block
  - 'draft'
  - 'published'
  - 'archived'
```

An enum type is therefore a union of literal types.

A literal type is a type whose only valid value is a specific literal. Since `!type` prefers identifier interpretation in ambiguous contexts, string literals must be quoted to disambiguate from type names:

```yaml
type: !type 'draft'    # string literal type: only "draft" is valid
type: !type "draft"    # equivalent
type: !type 42         # number literal type: only 42 is valid (unambiguous)
type: !type true       # boolean literal type: only true is valid (unambiguous)
```

A resource property declared with a literal type can only hold that exact value:

```yaml
# schema
properties:
  version:
    type: !type 1 # version must always be 1
  status:
    type: !type "draft" # status must always be "draft"

# resource
version: 1
status: draft
```

An enum type is therefore shorthand for a union of string literal types.

## TDR Explicit Type Tags

A value can carry an explicit type tag to override inference or disambiguate. The available tags are: `!string`, `!number`, `!boolean`, `!date`, `!link`. Any tagged value is an expression and can use operators, property references, and built-in functions:

```yaml
first_name: !string Bob
birth_date: !date 1990-07-04
count: !number 42
active: !boolean true
author: !link bob.tdr
full_name: !string self.first_name + " " + self.last_name
reviewer: !link self.default_reviewer
```

Expressions can reference:

- Other properties on the same resource: `self.first_name`.
- Properties on linked resources: `self.author.first_name`.
- Built-in functions.

## TDR Schema

A schema file self-identifies by setting `$type: schema`. It defines the shape of resources that reference it: what properties they have, their types, constraints, and default values. Each property supports the following fields:

- `type`: the type of the property, as a `!type` expression.
- `required`: whether the property must be present on the resource (default: `false`).
- `default`: a default value used when the property is absent. Wrap in `formula()` to derive the default from other properties.
- `values`: the allowed values for `!type enum` properties.

```yaml
---
$type: schema
properties:
  first_name:
    type: !type string
    required: true
  birth_date:
    type: !type date
  tags:
    type: !type list[string]
  status:
    type: !type enum
    values:
      - draft
      - published
      - archived
    default: draft
  full_name:
    type: !type string
    default: !string formula(self.first_name + " " + self.last_name)
  author:
    type: !type link[person]
---
```

`Schema` itself is a built-in schema, defined by the Typedown processor. It is typed by itself.

## TDR Markdown

The body of a `.tdr` file is written in TDR Markdown, an extension of standard Markdown with Typedown-specific syntax.

### Headings

Headings use the standard `#` syntax:

```markdown
# Heading 1

## Heading 2

### Heading 3
```

### Code

Inline code uses backticks. Code fences use triple backticks with an optional language tag:

````markdown
`inline code`

```python
print("hello")
```
````

### Blockquotes

```markdown
> This is a blockquote.
```

### Math

Inline math is wrapped in `$`. Block math is wrapped in `$$$`:

```markdown
The formula is $E = mc^2$.

$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$
```

### Tables

Tables use the standard Markdown pipe syntax:

```markdown
| Name  | Age |
| ----- | --- |
| Alice | 30  |
| Bob   | 25  |
```

### Lists

Bullet lists use `-` or `*`:

```markdown
- item one
- item two
  - nested item
```

Ordered lists use a number followed by `.`:

```markdown
1. first
2. second
   1. nested
```

Toggle lists use `>-`. The item is collapsed by default and can be expanded:

```markdown
>- Summary line
  Content shown when expanded.
```

### Callout Blocks

Callout blocks use `:::` with an optional type label:

```markdown
::: note
This is a note.
:::

::: warning
This is a warning.
:::
```

### Multimedia

Multimedia embeds images, video, audio, and iframes using the standard Markdown image syntax. The type is inferred from the URL or file extension:

```markdown
![alt text](image.png)
![demo](video.mp4)
![podcast](audio.mp3)
![embed](https://www.youtube.com/embed/dQw4w9WgXcQ)
```

### Links

Standard Markdown links are supported. Links can point to external URLs or to other `.tdr` files by filename:

```markdown
[Anthropic](https://anthropic.com)
[Bob](bob.tdr)
```

### Footnotes

Footnotes are declared in a `:::footnote` block. Keys are arbitrary identifiers, not indices: the rendered order is determined by appearance in the text, not the key name. Footnotes are referenced with `[^key]`:

```markdown
This is a claim.[^my_claim]

:::footnote
my_claim: This is the footnote text.
:::
```

### Bibliography

Bibliography entries are declared in a `:::bibtex` block. Entries are cited in the body with `[@key]`:

```markdown
Knuth described this in detail [@knuth1984].

:::bibtex
@book{knuth1984,
author: Donald Knuth,
title: The TeXbook,
year: 1984
}
:::
```

### String Interpolation

Any text in the body can embed expressions using `${}`. Expressions can reference the resource's own properties or traverse links:

```markdown
This note was written by ${self.author.first_name} ${self.author.last_name}.

Status: ${self.status}.
```
