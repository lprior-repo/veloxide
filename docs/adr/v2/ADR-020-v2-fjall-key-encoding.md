# ADR 020 (v2): Fjall Key Encoding Collisions

## Status
Accepted

## Context
Fjall is an LSM-tree and operates on raw byte slices. If we encode the event log sequence numbers or timer timestamps as strings (e.g., `instance_abc:9`, `instance_abc:10`), Fjall's lexicographic sorting will place `abc:10` *before* `abc:9` because `1` comes before `9` in ASCII.
If events or timers are sorted incorrectly, rehydration and sleep wakeups will be fundamentally broken and corrupt the state machine.

## Decision
All numeric components of a Fjall key must be encoded using **fixed-width, big-endian binary encoding**.

### Key Formats
1. **Events Partition:**
   `[instance_id_bytes][sequence_u64_be]`
   Using `u64::to_be_bytes()` guarantees that sequence `10` sorts correctly after sequence `9`.

2. **Timers Partition:**
   `[timestamp_u64_be][instance_id_bytes]`
   Using big-endian timestamps ensures that the background reanimator loop can safely scan from `[0u64_be]` to `[current_time_be]` to find exactly the timers that have expired, in correct chronological order.

## Consequences
- **Positive:** Mathematically perfect range scans and chronological replay.
- **Positive:** Performance optimization (comparing 8 bytes is faster than parsing string representations).
- **Negative:** Keys are not human-readable in raw database dumps, requiring the CLI to provide custom formatting for debugging.