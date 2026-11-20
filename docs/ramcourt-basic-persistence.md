# Updated Benchmarks
My recent merge to main branch for RamCourt gave it basic crash resilience. But there is still more to do. 

RamCourt Benchmark from CLSTR tests after giving it crash-resilience via replicating entire kv-state into a json file:
~ 3.5ms per batch update (10 keys) == ~0.35ms per update = ~350us per update
RAMCloud 80-node uses 13.5us for writes, 5us for reads. 350 to 13.5 us gap for me...

# Disparity Explanation

Big reason for the performance gap in RamCourt: It persists ENTIRE ram into storage on every write, including log cleaner writes.

If we want faster write speeds without sacrificing crash resilience, there's some modifications needed. A few that come to mind:
- WAL (Write Ahead Log), Make any mutable Log strictly Append-Only (Log is currently modify-anywhere), with a Compactor to create smaller new logs when data is being accessed later. It's Faster to append a small snippet to file than update the whole file, reduces call times for writes while not sacrificing the persistence/crash-resilience. Also improves reliance if server shuts off mid-write.
- fsync() and other system calls that flush in-core data to disk, likely can be used in conjunction with WAL. Deters interruptions when sync-ing to file, although our plans are to also include resilience to persist-interruptions also.

# Path towards Improved Performance

Path to WAL and fsync:
- Modify RAM logs for RamCourt to be either immutable or strictly-append. Compactor/Cleaner might still want to lock resources to create new logs but cannot modify existing ones (beyond appends). That part is TBD.
- Mimic RAM logs to ROM. This will ease maintance of the tool. We're aiming for correctness over speed for RamCourt, so ease of maintenance should be favored when possible.

Still TBD but part of proper WAL crash-resilience:
- Backup selection of data to other servers.
- Health checks of RamCourt servers and auto-detection of downed servers.
- Coordination of backup recovery.
- Tests that inject failures at specific moments of RamCourt server behavior & verify data does not get lost. (AKA test the damn thing you're most afraid of that can go wrong!!!)

Tangential to the Single-Key Write-speed Benchmark but still necessary for RamCourt:
- Zombie Detection (server went down and fails to realize it's not part of cluster anymore, risking incorrect updates for the cluster) and the related Zombie Handling.
- Transaction support, including Transactions for Multiple Keys saved on different servers
- Load-Balanced Ownership Transfers of Data to Reduce Client access bandwidth
- Table schemas with corresponding attribute updates: Eg, suppose in table a you declare a schema w attributes c,d,e, and you don't want to update the entire value for key b, just the attribute c, you should be able to write to a:b:c instead of a:b (version verification at least on a:b:c is STILL a requirement for updates to RamCourt)

Yes, I improved dataplane speed for the Router at Stateless by 200-300x, without sacrificing correctness. It didn't happen overnight. It involved careful integration tests and benchmarks, and a supporting team that shared great ideas to try, and the related collaborations. I'm effectively planning to go about RamCourt in a similar manner.
