# surrealdb-rust-wasm-repro

Minimal reproduction of memory increase and transaction overlapping using IndexedDB in Rust with WASM and SurrealDB

## Usage

Release Build using:

```bash
wasm-pack build --target web --out-name index
```

Release Build using:

```bash
wasm-pack build -- dev--target web --out-name index
```

Run using e.g. [http-server](https://www.npmjs.com/package/http-server) by calling:

```bash
http-server -c-1 .
```

in the root of the repository and navigating to [http://localhost:8080](http://localhost:8080).

There are two buttons, both use SurrealDB with indexeddb via Rust and WASM to create and select a few messages.

The first one creates a new db connection for every query, which very quickly increases the memory (~2 mb per query).

The second one uses a shared db connection, but runs into a transaction overlapping issue, which triggers a `TransactionInactive`
error in Firefox and hangs the WASM execution completely in Chrome.
(sometimes, you need to press a few times to trigger this, especially in a release build - you'll see that it happened in Firefox with an error log and in Chrome, if the console stops logging numbers)

