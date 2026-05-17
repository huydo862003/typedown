# Typedown Abstract Model

This document specifies the abstract model of Typedown, independent of any serialization format.

The model is inspired by the [property graph model](../research/graph-database/graph-databases-book/2-concept-property-graph-model.md) from graph databases and [RDF](../research/web-3/technologies/rdf.md) from the semantic web.

For a complete, opinionated serialization of the resource graph, see [TDR](./tdr.md) (how individual resources are serialized as markdown with YAML frontmatter) and [Typedown Vault](./typedown-vault.md) (how a collection of TDR files is organized on disk). Note that the vault is purely an organization convention for structuring TDR files on disk and has no meaning in the abstract model itself.

## Resource Graph

A Typedown project is a graph of [resources](#resources), which act as nodes. The graph model makes it natural to traverse connections, follow references, and query by structure rather than by table shape.

The graph is directed with **named, typed edges**: every [link](#links) has a source resource, a target resource, a name (what the relationship means), and a type. Nodes are also typed via [meta-resources](#meta-resources).

Links can be traversed in both directions via [forward links](#forward-links) and [back links](#back-links).

## Resources

A resource is the fundamental unit in Typedown. It represents any entity: a note, a person, a book, a tag, a project. Every resource has a unique identifier (URI) that remains stable regardless of how the resource is displayed or serialized.

Everything attached to a resource is a [**property**](#properties). Properties differ only in what their value is:

- A scalar value (e.g. a name, a date, a number (snake_case keys: `first_name`, `birth_date`)).
- A [**link**](#links) to another resource, forming an edge in the graph.
- One or more mandatory [**meta-resource**](#meta-resources) references, typing the resource itself.

## Meta-resources

A meta-resource is a resource that describes other resources. Where a regular resource represents content (a note, a person, a book), a meta-resource represents management structure: it governs how resources are organized, typed, and validated, not what they say. Think of it as the schema layer sitting above the content layer, concerned with shape and rules rather than meaning.

Currently, there is only one meta-resource: `Schema`. A `Schema` describes the shape of a resource, i.e. what properties it has and what their types are.

A resource can reference multiple `Schema`s, inheriting the shape of each. `Schema`s themselves support inheritance: a `Schema` can extend one or more other `Schema`s, combining their property definitions. The effective shape of a resource is the union of all its `Schema`s and their ancestors.

`Schema` is itself a resource. It is defined built-in and typed by itself.

## Properties

A property is a named value attached to a resource. Every property has:

- A **name**: identifies the property on the resource.
- A **value**: one or more values of a supported type.

Supported value types are:

- `string`
- `number`
- `boolean`
- `date`
- `enum`: a value from a fixed set of options.
- `link`: a reference to another resource, forming an edge in the graph (see [Links](#links)).
- `list[T]`: a list of values of type `T`.
- `record[K, V]`: a homogeneous mapping from keys of type `K` to values of type `V`.
- A fixed-key record: a mapping where each named key has its own independently typed value.

Whether a property is required or optional, and any constraints on its values, are enforced by the resource's [Schema](#meta-resources).

A property value can be a static value or an **expression**. The two are interchangeable: any property that holds a value can hold an expression instead. Expressions are evaluated lazily on read, and can reference:

- Other properties on the same resource.
- Properties on linked resources, traversing the graph.
- Built-in functions.

## Labels

A label is a human-friendly name for a resource. It is itself a property, but one with special semantics: it exists purely for display and identification in the UI rather than for data storage or querying.

A resource can have multiple labels, e.g. one per language. A `Schema` defines how the label is computed for resources of that type, either by pointing to an existing property (e.g. `name`) or by specifying a template that interpolates multiple properties (e.g. `"{firstName} {lastName}"`). The label therefore changes whenever the underlying properties change.

## Links

A link is a property whose value is a reference to another resource. It forms a directed edge in the resource graph, connecting exactly two resources.

A link is defined by two things:

- A **name**: the property name on the source resource (e.g. `author`, `tag`, `relatedTo`).
- A **target schema**: the `Schema` that the target resource must conform to. This constrains what kind of resource can be linked to.

Links do not carry properties of their own. If a relationship needs its own data (e.g. a role, a date, a weight), the recommended approach is to model it as a new resource with its own `Schema`, and link both parties to that resource.

### Forward Links

A forward link is a link as seen from the source resource: it is a named property whose value is the target resource.

### Back Links

A back link is the same link as seen from the target resource: it exposes the incoming references to a resource. Back links are not stored separately: They are derived by traversing the graph in reverse. A `Schema` can define which back links are surfaced on a resource and under what name.
