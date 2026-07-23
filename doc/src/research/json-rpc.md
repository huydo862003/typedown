# JSON-RPC 2.0

References: [www.jsonrpc.org](https://www.jsonrpc.org/).

## Background

This section walkthrough the background knowledge required to understand JSON-RPC.

### JSON - Serialization Format

JSON (Javascript Object Notation) is simply a serialization format that is:

- Portable.
- Readable.
- Capable of representing most plain values.

3 basic components that universally present in almost any programming languages:

- Scalars: true, false, null, string, number.
- Objects: collections of name/value pairs.
- Arrays: ordered lists of values.

With these properties, JSON is usually used as a data-interchange format, typically seen in web APIs.

### RPC - Just Some Wrapper Functions

RPC (remote procedure call) is a type of client-server API, in which a process triggers the execution of a procedure in the context of a different address space.

As far as I know, RPC consists of 3 parts:

- The server defines and publishes some well-known methods.
- The client which will want to trigger these methods.
- Some stubs that represent the remote methods, and the client will call these stubs like normal functions.

Essentially, from the point of view of the client, RPC is a just a normal function, which wraps around some logic to notify the remote processes.
