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

**Open questions:** Has anyone hit a case where version ordering mattered beyond recovery replay? And how do you order a recreated key against its tombstone? I’d like to hear your war stories on building key-value storage.
