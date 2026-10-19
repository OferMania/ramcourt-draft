# Why RamCourt’s versions are random UUIDs, not counters
RamCourt is a key-value store that grew out of my years working on [RAMCloud](https://github.com/PlatformLab/RAMCloud), which I got to know at Stateless. I kept the decisions that worked and changed some others. Two of those changes are related:

Every update and delete must present the key’s current version, so a client has to know a key’s state before changing it. RAMCloud offers similar checks through reject rules, which are optional and chosen by the client. In RamCourt the check is mandatory and enforced by the server.

That version is a random 128-bit UUID (122 random bits and 6 fixed bits) instead of RAMCloud’s incrementing 64-bit integer.

In RAMCloud, one number does two jobs: it guards client writes, and it orders copies of an object during recovery. RamCourt splits them. The client-visible UUID is only a guard, and ordering will come from a separate internal sequence. Here is why the guard shouldn’t be a counter:
- **Counters make it easy to confuse one key’s version with another’s.** In RAMCloud, a new key takes its starting version from a table-wide counter, while an update increments that key’s own version separately from that counter. So if key1 is at 7, key2 is created at 8, and key1 is then updated to 8, a client holding key2’s version can successfully update key1. RamCourt needs to make a mistake like that as unlikely as possible.
- **Small numbers are easy to guess.** Even if a version could never be valid for two different keys, low, densely packed numbers mean a wrong guess is likely to match. A key that has been updated twice sits at version 3, so a client that has lost track can just try 3. With 122 random bits, guessing isn’t a strategy.
- **Only equality was ever needed.** In my use of RAMCloud, every time I enforced a version through reject rules, I only needed an exact-match check. So the client-visible version doesn’t need to increase, and random selection suffices. A mistaken version matches with probability 2^-122 per attempt.

**What the UUID doesn’t replace.** Recovery replay needs to decide which copy of a key wins, and tombstones need to keep deleted keys from returning. So RamCourt will keep an internal, linearly increasing 64-bit sequence, sent with every backup write once those features exist. I am undecided on whether this should be per key or table-wide. After a delete and recreate, the new entry has to sort after the tombstone. [Rumble’s dissertation, Chapter 6 (‘Tombstones and Object Delete Consistency’)](https://web.stanford.edu/~ouster/cgi-bin/papers/RumblePhd.pdf)

**Tradeoffs.** The UUID costs 16 extra bytes per object. The primary must also send it with every backup write, and recovery replay must preserve it instead of regenerating it. Neither is built yet.

**Open questions:** Has anyone hit a case where version ordering mattered beyond recovery replay? And how do you order a recreated key against its tombstone? I’d like to hear your war stories on building key-value storage. Replies welcome in [Discussions](https://github.com/OferMania/ramcourt-draft/discussions/7).

## Addendum (October 2026): what readers told me

Since publishing this, I've had substantive replies on Reddit, the Software Internals Discord, and LinkedIn. They changed my design notes in four ways. Thanks to everyone who took the time. Nothing in this addendum is implemented yet.

### 1. A UUID can't order anything, so ordering needs its own sequence

A Reddit commenter pointed out that a random client-visible version can't say a recreate happened after a delete, because nothing about a UUID says it happened later. They recommended a server-assigned, per-key monotonic counter that the client doesn't choose. They also warned that a table-wide counter answers a different question (the order of the replication log) and would let one hot key serialize the whole table.

That matches the split in the post: the UUID is the guard, and ordering comes from a separate internal sequence. What's still open is the scope of the sequence:

- **Per key.** Unrelated keys never share an order. The cost is that a recreate has to read the old counter, so the counter has to outlive the delete. I'd have to retain state for deleted keys until no older copy can resurface, which is the tombstone garbage-collection problem.
- **Table-wide.** No retained per-key state. The cost is shared state that must survive restarts and move with ownership. A Discord reader who has only used global sequence counters didn't think the increment was much of a scaling hurdle, and on a single primary I agree. The serialization concern is real for a replicated log, where the whole group shares one order.
- **Log index.** If replication ends up as a Raft-style log, a delete followed by a recreate is just two entries in order, and the question becomes how large a group should be. I'm leaning that way but haven't decided.

### 2. Retries need their own mechanism

When a response is lost and the client retries, the retry is rejected because the version has already changed. The client can't tell whether its write applied or someone else's did. The version guard can't fix this, because it answers a different question.

The fix is a request ID with two parts: a random client UUID per session, plus a counter the client increments on every request. The server records each completed request's ID and response per client UUID. A retry carrying the same ID gets the recorded response, including the new version, instead of a conflict. The counter also lets a client acknowledge everything up to N, so the server can discard those records. A restarted client starts a new session and never reuses an old (UUID, counter) pair.

### 3. Three jobs, three mechanisms

| Job | Mechanism | Assigned by |
|---|---|---|
| Prove the client saw the current state | Random UUID version | Server |
| Order delete vs. recreate, and recovery replay | Internal sequence (scope undecided) | Server |
| Make retries safe | Client UUID + request counter | Client |

One suggestion was to make the UUID a client-sent idempotency token and let a server-assigned per-key counter do the ordering. That is close to what I landed on for idempotency. The difference is the guard. If clients presented a per-key counter as their guard instead of a server-issued UUID, we'd be back to small, densely packed numbers that a mixed-up client can match by accident, which is the problem that motivated this post. So I'm keeping the random UUID as the guard and giving the client-sent ID its own field.

### 4. Ownership and consistency model

One reader suggested hashing an identifier to a single owner node that queues requests. That is close to how RAMCloud assigns keys to servers, and it's the model RamCourt assumes. It serializes writes on the server, but it doesn't replace the version guard: two clients that both read the same value will both write, and without a guard the second silently overwrites the first. What to hash is the key, since a version changes on every write and can't pick the node. Static hashing also balances load only when traffic is uniform, which is why I want to explore moving ownership of hot keys.

Another reader, who worked on an eventually consistent store, described a node-local counter plus a vector clock. Vector clocks address concurrent writes to the same key on different replicas, which RamCourt is designed to avoid: it has one owner per key at a time, and the mandatory version check settles conflicts at write time. They thought the counter probably had to be persisted but couldn't recall for sure. That matters for me too, since a restarted node must not reissue lower numbers.

### Still open

- How long to keep deleted-key state, if the sequence is per key
- Where the sequence lives, and how it moves when ownership moves
- How long completed-request records are kept, and when a client's session expires
- How an idempotent retry should behave if the key has expired in between

Corrections and war stories welcome in [Discussions](https://github.com/OferMania/ramcourt-draft/discussions/7).
