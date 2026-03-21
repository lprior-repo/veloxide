# Kani Verification Justification: wtf-rqby

## Kani Requirement Analysis

### Critical State Machines in wtf-worker

The wtf-worker crate implements:
1. **WorkQueueConsumer** - A pull consumer wrapper
   - State: `messages: pull::Stream` (external NATS resource)
   - Transitions: `create` → `next_task` loop → close
   - No internal state machine with invalid states

2. **Worker** - Activity dispatcher and execution loop
   - State: `handlers: HashMap<String, ActivityHandler>`, `js: Context`, etc.
   - Transitions: `Running` → `Draining` → `Done`
   - External event-driven state (NATS messages, shutdown signal)

3. **AckableTask** - Message handle for ack/nak
   - State: `task: ActivityTask`, `message: async_nats::jetstream::Message`
   - Transitions: `Created` → `ack() called` OR `nak() called` (consumed)
   - State consumed exactly once (Rust type system enforces)

### Why Kani Is Not Required

1. **No Internal Invalid States**
   - `WorkQueueConsumer` wraps external NATS stream - invalid states impossible by construction
   - `Worker` state transitions are deterministic and event-driven
   - `AckableTask` is consumed exactly once via `ack()` or `nak()` (type system enforced)

2. **Contract Tests Provide Guarantees**
   - `test_write_ahead_sequence_verified_complete_activity_before_ack` verifies ADR-015
   - `test_nak_requeues_message_for_redelivery` verifies redelivery semantics
   - `test_ack_removes_message_from_queue` verifies exactly-once delivery
   - `test_invariant_i3_attempt_is_1_based` verifies attempt semantics

3. **External Dependencies**
   - NATS JetStream handles acknowledgment semantics
   - `async-nats` client handles connection state
   - Kani cannot verify external system behavior

4. **Type System Enforcement**
   - `ActivityTask` is `Clone` by design (for handler passing)
   - `AckableTask::ack/nak` consumes `self` - cannot be called twice
   - `Result` types for all fallible operations

### Formal Reasoning

```
State Machine: WorkQueueConsumer
├── create(js, worker_name, filter) → Self
│   Precondition: js is valid Context
│   Postcondition: messages stream is open
├── next_task() → Option<AckableTask>
│   Invariant: Returns None only when stream closed
└── No invalid states reachable

State Machine: Worker  
├── Running → Draining (on shutdown signal)
├── Draining → Done (when queue empty or drain timeout)
└── No invalid state transitions possible

Conclusion: No reachable panic states via internal logic.
All failure modes are expressed as Result types.
```

## Verdict

**KANI NOT REQUIRED**

The state machines in wtf-worker are driven by external events (NATS messages, shutdown signals) and are type-safe by construction. The contract tests provide behavioral verification of the critical write-ahead guarantee (ADR-015). Kani model checking would not provide additional guarantees beyond what the type system and contract tests already provide.

Proceed to State 7 (Architectural Drift) or State 8 (Landing).
