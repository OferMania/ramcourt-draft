# ramcourt-draft
Key Value Storage Implementation with Court Credential Verification

## Local Testing

Run `cargo build` to build code.
Run `cargo run` to bring up server onto localhost

From a different terminal window, you can interact with localhost

Healthcheck the server:

curl -i -X GET http://localhost:8080/health

Set key "abcd" to value "Hello World":

curl -i -X PUT http://localhost:8080/kv/abcd -H "Content-Type: application/json" -d "{\"value\": \"Hello World\", \"duration_secs\": 180}"

Get the value for key "abcd":

curl -i -X GET http://localhost:8080/kv/abcd

Record the version returned here if you WANT to update key "abcd", suppose it's 9bffaa74-fcf0-4020-a100-ff16ffab4003

The subsequent update REQUIRES you know the version of what you are changing. RamCourt REQUIRES clients to have the correct assumptions prior to updating.

Now let's change the value for the key using our version.

curl -i -X POST http://localhost:8080/kv/abcd -H "Content-Type: application/json" -d "{\"value\": \"Goodbye World\", \"duration_secs\": 360, \"version\":\"9bffaa74-fcf0-4020-a100-ff16ffab4003\"}"

Record the version that comes from the output. Yes that version will be different than what you passed in. If you forget the version, you can always find it out from a GET like before:

curl -i -X GET http://localhost:8080/kv/abcd

Suppose you find out the updated version is 264f4578-d401-4acd-865f-31dae9f222d4

To delete key "abcd", you gotta know that version. Then you do:

curl -i -X DELETE http://localhost:8080/kv/abcd -H "Content-Type: application/json" -d "{\"version\":\"264f4578-d401-4acd-865f-31dae9f222d4\"}"

Now notice that ANYTIME you perform GET's, PUT's, or DELETE's, you get back a response with the table_version. (That's different from version, version is specific to a key-value entry, table_version is the version for the whole table) 

Now suppose you forgot the table_version or are undecided on a query to find it, here's a simple way to get it:

curl -i -X GET http://localhost:8080/table

Suppse table_version is ff17b1fc-3e1b-4fec-83d9-22f30ad18efa

To clear all keys in the table, you do:

curl -i -X DELETE http://localhost:8080/table -H "Content-Type: application/json" -d "{\"table_version\":\"ff17b1fc-3e1b-4fec-83d9-22f30ad18efa\"}"

# Distributed Key-Value Store Challenge

Build a distributed key-value store from scratch using the Raft consensus algorithm.

## Stages

1. **[http-api](https://clstr.io/kv-store/http-api/)** - Store and Retrieve Data
2. **[persistence](https://clstr.io/kv-store/persistence/)** - Data Survives SIGTERM
3. **[crash-recovery](https://clstr.io/kv-store/crash-recovery/)** - Data Survives SIGKILL
4. **[leader-election](https://clstr.io/kv-store/leader-election/)** - Cluster Elects and Maintains Leader
5. **[log-replication](https://clstr.io/kv-store/log-replication/)** - Data Replicates to All Nodes
6. **[log-compaction](https://clstr.io/kv-store/log-compaction/)** - System Manages Log Growth
7. **[membership-changes](https://clstr.io/kv-store/membership-changes/)** - Add and Remove Nodes One at a Time
8. **[joint-consensus](https://clstr.io/kv-store/joint-consensus/)** - Add and Remove Nodes in Bulk

## Getting Started

1. Read the requirements for the current stage (linked above)
2. Update the `Dockerfile` if required
3. Run `clstr test` to verify your implementation
4. Run `clstr next` to advance when tests pass

## Resources

- [Challenge Overview](https://clstr.io/kv-store/)
- [How clstr Works](https://clstr.io/how-it-works/)
- [CLI Guide](https://clstr.io/guides/cli/)
- [CI/CD Setup](https://clstr.io/guides/ci-cd/)

Run `clstr --help` to see all available commands.
