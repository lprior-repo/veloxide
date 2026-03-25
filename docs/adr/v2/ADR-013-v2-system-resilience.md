# ADR 013 (v2): System Resilience (Thundering Herds, Disk Watchdogs)

## Status
Accepted

## Context
When deploying a workflow engine on a single node, operational physics dictate failure modes. 
1. **The Thundering Herd:** If the server crashes with 5,000 active instances, upon restart, the engine will instantly attempt to replay and resume all 5,000 instances, pinning the CPU and likely triggering another crash.
2. **The Disk Deadlock:** If the SSD hits 100% capacity, the `DbWriterActor` cannot flush `fjall` events. The entire system deadlocks silently.
3. **Clock Skew:** Relying solely on `fire_at` timestamps for hibernation wakeups breaks if the server's NTP clock jumps backwards or forwards.

## Decision
We implement a three-tiered System Resilience protocol.

### 1. Crash Recovery Startup Throttle
On startup, the Engine does not instantly resume all in-flight instances.
- The Engine reads the instances from `fjall` and places them in a recovery queue.
- It processes the queue in configurable batches (e.g., `recovery_batch_size = 50`).
- Instances are prioritized based on time-in-flight (instances closest to completion are prioritized to drain the system).

### 2. Disk Space Watchdog
A background Tokio task runs every 30 seconds to check filesystem capacity.
- If free space drops below a critical threshold (e.g., 1GB), the Engine enters **Degraded Mode**.
- In Degraded Mode: The Engine stops accepting new workflows (`HTTP 503`), pauses non-critical timers, and prioritizes finishing in-flight instances to clear space.
- A 10-second flush timeout is applied to the `DbWriterActor`. If a flush times out, the Engine initiates a graceful shutdown rather than deadlocking in a corrupt state.

### 3. Dual-Clock Timer Verification
To survive clock skew, the Engine records both an absolute timestamp and a monotonic duration when a workflow hibernates.
- The `TimerScheduled` event contains: `fire_at` (Absolute) and `duration_ms` (Monotonic relative to insertion).
- The reanimator loop verifies `fire_at <= Utc::now() OR elapsed_since_set >= duration_ms`.

## Consequences
- **Positive:** The system degrades gracefully under extreme duress instead of crashing or corrupting state.
- **Positive:** Time-travel bugs caused by NTP syncs are eliminated.
- **Negative:** Additional background tasks to manage (Watchdog, Thundering Herd throttle).