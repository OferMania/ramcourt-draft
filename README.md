# RamCourt

A Rust-based, strongly-consistent, fault-tolerant distributed key-value
store, inspired by RAMCloud. RamCourt's defining property: every read,
write, and delete requires clients to supply the correct current version
(or table_version, for whole-table operations) before a mutation is
permitted — a mandatory version-verification model, not an optional
client-side check. This prevents accidental updates based on stale
assumptions about the data's current state.

RamCourt is currently a single-node, in-memory key-value store. See
[Known Limitations](#known-limitations) for what's not built yet.

## Local Testing

Run `cargo build` to build the code.
Run `cargo run` to bring up the server on localhost.

From a different terminal window, you can interact with localhost:

Healthcheck the server:

```
curl -i -X GET http://localhost:8080/health
```

Set key "abcd" to value "Hello World":

```
curl -i -X PUT http://localhost:8080/kv/abcd -H "Content-Type: application/json" -d '{"value": "Hello World", "duration_secs": 180}'
```

Get the value for key "abcd":

```
curl -i -X GET http://localhost:8080/kv/abcd
```

Record the version returned here if you want to update key "abcd" — suppose it's `9bffaa74-fcf0-4020-a100-ff16ffab4003`.

The subsequent update REQUIRES you know the version of what you are changing. RamCourt requires clients to have the correct assumptions prior to updating.

Now let's change the value for the key using our version:

```
curl -i -X PUT http://localhost:8080/kv/abcd -H "Content-Type: application/json" -d '{"value": "Goodbye World", "duration_secs": 360, "version":"9bffaa74-fcf0-4020-a100-ff16ffab4003"}'
```

Record the version that comes from the output (note: do not get confused with table_version, which is a different field). Note that the version that is output will be different than what you passed as input. If you forget the version, you can always find it out from a GET like before:

```
curl -i -X GET http://localhost:8080/kv/abcd
```

Suppose you find out the updated version is `264f4578-d401-4acd-865f-31dae9f222d4`.

To delete key "abcd", you need to know that version:

```
curl -i -X DELETE http://localhost:8080/kv/abcd -H "Content-Type: application/json" -d '{"version":"264f4578-d401-4acd-865f-31dae9f222d4"}'
```

Anytime you perform GETs, PUTs, or DELETEs, you get back a response with `table_version`. That's different from `version` — `version` is specific to a single key-value entry, `table_version` is the version for the whole table.

If you forget the `table_version`, here's a simple way to find it:

```
curl -i -X GET http://localhost:8080/table
```

Suppose `table_version` is `ff17b1fc-3e1b-4fec-83d9-22f30ad18efa`.

To clear all keys in the table:

```
curl -i -X DELETE http://localhost:8080/table -H "Content-Type: application/json" -d '{"table_version":"ff17b1fc-3e1b-4fec-83d9-22f30ad18efa"}'
```

## Testing

To quickly setup clstr-rc, which is clstr adapted for RamCourt's version verification, then run this command:

```
chmod +x setup_clstr_rc.sh
./setup_clstr_rc.sh
```

Note that `main` branch of clstr-rc is used by default. To instead use a different branch in clstr-rc, you can do:

```
chmod +x setup_clstr_rc.sh
CLSTR_BRANCH='some-branch' ./setup_clstr_rc.sh
```

After setup_clstr_rc.sh is ran, there should be a clstr tool present. But first, let's make sure RamCourt is built:

```
cargo build
```

To run RamCourt against clstr tests, you can do:

```
./clstr test
```

Please note that RamCourt works against the first set of tests in clstr at the moment (http_api), but doing `./clstr next` will advance clstr internal-state to a set of tests that do not yet work for RamCourt.

RamCourt's HTTP API is tested using clstr-rc, a fork of [clstr](https://github.com/clstr-io/clstr), adapted for RamCourt's version-verification response format: [clstr-rc](https://github.com/OferMania/clstr-rc). See that repo for build/setup instructions.

## Known Limitations

- Single global table only; no multi-table support yet.
- Single node only; no backup propagation or replication between nodes yet.
- Recovery; if a node goes down, no attempt is made to recover its data from other nodes and make a different node primary owner for that data.

## Inspiration & Tooling

The HTTP API and in-memory store stages were built following the stages
outlined in [clstr.io's key-value store challenge](https://clstr.io/kv-store/),
with reference to [this blog post on building an in-memory KV store](https://stephenmwangi.com/blog/in-memory-key-value-store/).
Implementation is independent, in Rust rather than the challenge's
reference Go implementation. Where the challenge's spec was silent,
notable original design choices include:

- Mandatory version-verification on every read, write, and delete —
  not part of the base challenge, and the defining property of RamCourt.
- `table_version`, naming chosen ahead of planned multi-table support
  (see Known Limitations).

Test tooling: [clstr-rc](https://github.com/OferMania/clstr-rc), a fork
of clstr adapted to verify RamCourt's version-verification response
format, including a 100-concurrent-write test confirming exactly one
writer succeeds under contention.

## License

[Apache 2.0](./LICENSE). See [CONTRIBUTING.md](./CONTRIBUTING.md) and
[CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md) for contribution guidelines.

