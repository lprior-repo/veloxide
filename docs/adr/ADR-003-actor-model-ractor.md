# ADR-003: Use ractor for Actor Model

## Status

Accepted

## Context

wtf-engine needs:

- **Isolation** - Each workflow instance runs independently
- **Supervision** - Parent monitors child lifecycle (restart on failure)
- **Message passing** - Workflows communicate via async messages
- **Concurrency** - 3 concurrent workflows without shared state
- **Fault tolerance** - Handle actor crashes gracefully

### Alternatives Considered

| Approach | Pros | Cons |
|---------|------|------|
| **ractor** | Erlang-style, supervision trees, async, Rust-native | Pre-1.0, smaller ecosystem |
| **async-std** | Async-first, familiar API | No actor model, no supervision |
| **tokio** | Production-grade runtime | No actor model, manual task management |
| **actix** | Web-focused | Not designed for workflow orchestration |
| **Custom actor** | Full control | Re-inventing the wheel |

### Why Not actix

- actix is primarily a **web framework**, not an actor framework
- Limited supervision capabilities
- Designed for HTTP request handling, not workflow orchestration

### Why Not Custom Actor

- Erlang/OTP supervision is a solved problem
- Building reliable supervision from scratch is extremely complex
- ractor provides battle-tested patterns

## Decision

We will use **ractor** for the actor model, with **MasterOrchestrator** as the top-level supervisor and **WorkflowInstance** as the per-workflow actor.

### Why ractor

1. **Erlang-style supervision** - `spawn_linked`, `SupervisionEvent`
2. **Async-native** - All handlers are `async fn`
3. **Rust-native** - No FFI, pure Rust
4. **Message passing** - `call!`, `call_t!`, `cast!` patterns
5. **Supervision trees** - Parent-child relationships with lifecycle events
6. **Process groups** - For fan-out parallelism

### Actor Hierarchy

```
MasterOrchestrator (root supervisor)
│
├── WorkflowInstance:checkout:inv_001
│       └── ActivityActor:charge_card (spawned as needed)
│       └── ActivityActor:send_email
│
├── WorkflowInstance:checkout:inv_002
│
└── WorkflowInstance:order:inv_003
```

### Core Patterns

```rust
// Spawn linked (child under supervision)
let (actor_ref, handle) = Actor::spawn_linked(
    Some("workflow-1".into()),
    WorkflowInstance::new(workflow_name.clone()),
    InstanceConfig { invocation_id, input, storage },
    myself.clone().into(),  // supervisor ActorCell
).await?;

// Supervision event handling
async fn handle_supervisor_evt(
    &self,
    myself: ActorRef<Self::Msg>,
    event: SupervisionEvent,
    state: &mut Self::State,
) -> Result<(), ActorProcessingErr> {
    match event {
        SupervisionEvent::ActorFailed(cell, error) => {
            // Restart policy: always, on_error with max_retries, or never
            self.restart_child(myself, cell.get_name().unwrap()).await;
        }
        _ => {}
    }
    Ok(())
}

// RPC with timeout
let result: Result<StepOutput, StepError> = call_t!(
    workflow_actor,
    InstanceMsg::ExecuteStep,
    30_000,  // 30 second timeout
    step_input
).await?;
```

## Consequences

### Positive

- Fault isolation (one workflow crash doesn't kill others)
- Clear supervision hierarchy
- Async-native (no blocking)
- Typed message passing (compile-time safety)
- Built-in timeout patterns

### Negative

- Additional dependency (ractor 0.15)
- Pre-1.0 stability (monitor closely)
- Learning curve for Erlang-style patterns

### Migration Path

If ractor proves unsuitable:
- Extract actor interface behind a trait
- Implement custom actor using tokio::spawn + channels
- Minimal surface area change required
