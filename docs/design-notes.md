# Disclaimer

RAMCloud claims come from these citations:

- https://ramcloud.atlassian.net/wiki/spaces/RAM/overview
- https://dl.acm.org/doi/epdf/10.1145/2806887
- https://web.stanford.edu/~ouster/cgi-bin/papers/RumblePhd.pdf
- https://web.stanford.edu/~ouster/cgi-bin/papers/rifl.pdf
- https://www3.dcc.fc.up.pt/~ines/aulas/1314/SDM/papers/MinCopysets.pdf
- https://github.com/PlatformLab/RAMCloud/blob/master/designNotes
- https://groups.google.com/g/raft-dev/c/wF-5xnylm2o?pli=1
- https://github.com/OferMania/ramcloud

# Design decisions intended to be different in RamCourt from RAMCloud

- Client-facing version is non-linear UUID
- Client-facing version is a core update requirement to change/delete the value. RAMCloud has optional reject-rules that mimics this behavior, but RamCourt makes this mandatory.
- subscriptions on RPC communication-layer, so clients can see when a key they subscribed to changed, and to what.
- Rack id and server id are required for a server on registering to RamCourt, preventing a server from designating a different server on the same rack as a backup. RAMCloud has a command-line argument (--useMinCopysets) for optional copyset-style replication groups, but RamCourt adds mandatory rack separation when choosing backups. The reasoning is, if power goes out on a full rack, and that’s the only rack where the data is saved, then that data might be entirely lost
- Table-schemas (with named fields and optimizations/pointers for flexible array fields), and the related attribute update API’s
- Table-schema migrations
- Table-cleanups, if a table is empty (because suppose all its keys expired), then after a certain amount of elapsed time, that table should be auto-removed.
- Automatic usage-driven Ownership-transfers of keys and tables to facilitate fair-shake client updates and minimize bandwidth
- Ownership-transfers of keys and tables to MINIMIZE server interactions needed for transactions <- This is actually a BIGGER motivator for RamCourt ownership transfers than the usage-driven reason. We still want to eventually have RamCourt support 2PC (2 phase commit), especially if the manner of a client-requested transaction is not easily fulfillable or complicates other interactions due to, say, requiring a super-popular key ownership transfer to an inconvenient server.
- Each write is potentially a Transaction (multikey write implies all versions provided must be valid for transaction to happen). RamCourt requires each transaction/operation to have mandatory versions.
- RamCourt behavior here is undecided, but I’m leaning towards, with backup-propagation behaviors, backup servers writing to storage before RAM, and master-server requiring FULL acknowledgements (RAM and storage) from all designated backup-servers before responding to client with a success response. This differs from RAMCloud’s behavior of backup server writing to RAM before storage, and master-server only requiring acknowledgement of the RAM-writes (without storage) from backup servers before responding to client with a success response. RAMCloud assumes its backups’ RAM buffers survive a power cut; RamCourt can’t assume that, which is a driving reason for this choice.

# Design decisions intended to be similar to RAMCloud
- Sequential internal versions for keys due to tombstone management per server (ownership transfers of keys might need to make adjustments, will need to think that part thru when I get to it)
- Coordinator load-balancing table/keys on different nodes
- Reads/Writes in Batches (2 Batched writes each on single key different from single transaction on 2 keys in this way: For batch one write might fail due to version and another write might succeed with valid version, they’re unrelated to each other. For transaction, write only occurs if versions for both keys are valid. Ie batches are like “unions” and transactions are like “intersections”.)
- Transactions are to be permitted across multiple tables, but I might for first-pass only implement single-table or single-tablet transactions (that rely entirely on one master node), before changing over to implementing 2PC for transactions across multiple tablets/tables/servers.
- Immutable Logs (except the currently updated Logs) and RAM/ROM representation of Logs matching as closely as possible, with Cleaning/Compactions of older logs.
- Reduction of server failure state-tracking. If server notices any data/network problems (especially with checksum hashes corresponding to log entries), it exits immediately, simplifying life-check management from coordinator and peer server nodes.

# Known Limitations for RamCourt
- A master server that is paused, partitioned, or restarts with a specific disk image risks serving stale data (it’s a Zombie). RAMCloud solves this by having a server detect failed pings to peers and having it follow-up by syncing with coordinator, who verifies if a server is registered or not. In the case of not, server learns it’s a zombie and exits. Also, after recovery begins, backups reject replication from the old master, which blocks zombie writes. I will need to think thru if RamCourt needs a different solution. So far, leaning towards keeping these approaches.
- Idempotency Problem: If a master crashes mid-replication, it might’ve finished applying the writes to backups, but client sees 409 for the write that actually succeeded. This is a known edge case for now.
