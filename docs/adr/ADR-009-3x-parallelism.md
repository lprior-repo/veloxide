# ADR-009: 3x Parallelism Enforcement

## Status

Accepted

## Context

wtf-engine runs on a **single machine** with **limited resources**. To prevent resource exhaustion:

- We need to limit **concurrent workflow executions**
- Default limit: **3 concurrent workflows**
- Should be **configurable**
- Enforcement must be at the **orchestrator level**

### Why 3?

- **Conservative default** for single-machine
- **Reasonable for development** - Most dev machines can handle 3 concurrent workflows
- **Configurable** - Production can increase to match hardware
- **Based on restate/temporal defaults** - Both suggest low concurrency for single-node

### Resource Considerations

| Resource | Per Workflow | 3 Concurrent |
|----------|--------------|--------------|
| Memory | ~10-50MB | ~30-150MB |
| CPU | Varies | Varies |
| Disk I/O | Journal writes | ~3x |
| Actor overhead | ~1MB | ~3MB |

## Decision

We will enforce **3x parallelism** at the MasterOrchestrator level with configurable override.

### Implementation

```rust
pub struct MasterOrchestrator {
    max_concurrent: usize,  // Default: 3
    storage: Arc<sled::Db>,
}

impl Default for MasterOrchestrator {
    fn default() -> Self {
        Self {
            max_concurrent: 3,
            storage: Arc::new(sled::open("wtf-engine.db").unwrap()),
        }
    }
}

impl MasterOrchestrator {
    pub fn new(max_concurrent: usize, storage: Arc<sled::Db>) -> Self {
        Self {
            max_concurrent,
            storage,
        }
    }
}
```

### Capacity Check

```rust
async fn handle(
    &self,
    myself: ActorRef<Self::Msg>,
    msg: OrchestratorMsg,
    state: &mut OrchestratorState,
) -> Result<(), ActorProcessingErr> {
    match msg {
        OrchestratorMsg::StartWorkflow { name, input, reply } => {
            // Capacity check
            if state.running_count >= self.max_concurrent {
                reply.send(Err(StartError::AtCapacity {
                    running: state.running_count,
                    max: self.max_concurrent,
                }))?;
                return Ok(());
            }

            // Spawn workflow instance
            let invocation_id = self.spawn_workflow(state, name, input).await?;

            // Increment counter
            state.running_count += 1;

            reply.send(Ok(invocation_id))?;
        }
        // ...
    }
    Ok(())
}
```

### Capacity Tracking

```rust
pub struct OrchestratorState {
    instances: HashMap<String, (String, ActorRef<InstanceMsg>)>,
    running_count: usize,
    completed_count: u64,
    failed_count: u64,
}

// Supervision handler decrements counter
async fn handle_supervisor_evt(
    &self,
    _myself: ActorRef<Self::Msg>,
    event: SupervisionEvent,
    state: &mut OrchestratorState,
) -> Result<(), ActorProcessingErr> {
    match event {
        SupervisionEvent::ActorTerminated(_) |
        SupervisionEvent::ActorFailed(_) => {
            state.running_count = state.running_count.saturating_sub(1);
        }
        _ => {}
    }
    Ok(())
}
```

### Configuration

```bash
# CLI flag
wtf serve --max-concurrent 10

# Environment variable
WTENGINE_MAX_CONCURRENT=10 wtf serve

# Config file
# wtf.toml
[server]
max_concurrent = 10
```

### When Capacity is Reached

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum StartError {
    #[error("at capacity: {running} workflows running (max {max})")]
    AtCapacity { running: usize, max: usize },

    #[error("workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}

// API response
{
    "error": "at_capacity",
    "message": "3 workflows running (max 3)",
    "retry_after_seconds": 5
}
```

## Consequences

### Positive

- **Prevents resource exhaustion** - System stays responsive
- **Simple implementation** - One counter check
- **Observable** - `running_count` exposes current load
- **Configurable** - Match hardware capacity

### Negative

- **Request queuing needed** - Clients must handle "at capacity" errors
- **Tuning required** - 3 may be too low for powerful machines

### Future Considerations

- **Priority queuing** - High priority workflows get slots first
- **Backpressure** - Caller waits instead of error
- **Autoscaling** - Not applicable for single-machine
