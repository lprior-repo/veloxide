# ADR-006: MasterOrchestrator + WorkflowInstance Actor Hierarchy

## Status

Accepted

## Context

wtf-engine needs to:

- Manage **multiple concurrent workflow instances**
- Enforce **capacity limits** (max 3 concurrent)
- Handle **workflow lifecycle** (start, signal, cancel, complete)
- Provide **fault isolation** (one workflow crash ≠ system crash)
- Support **workflow registry** (find running workflows by ID)

### Design Constraints

1. **3x parallelism** - Only 3 workflow instances can run simultaneously
2. **Actor supervision** - Failed workflows should be restartable or cleanly terminated
3. **Registry** - Map invocation_id → workflow actor
4. **Signals** - External events must route to correct workflow instance

## Decision

We will use a **two-level actor hierarchy**:

```
MasterOrchestrator (root)
├── Maintains capacity counter
├── Maintains workflow registry (invocation_id → actor)
├── Spawns WorkflowInstance actors
└── Handles SupervisionEvent from children

WorkflowInstance (one per workflow)
├── Owns workflow DAG (petgraph)
├── Owns journal cursor
├── Executes steps
└── Reports to MasterOrchestrator
```

### MasterOrchestrator

```rust
pub struct MasterOrchestrator {
    max_concurrent: usize,
    storage: Arc<sled::Db>,
}

pub struct OrchestratorState {
    // invocation_id → (workflow_name, actor_ref)
    instances: HashMap<String, (String, ActorRef<InstanceMsg>)>,
    running_count: usize,
}

#[derive(Message)]
pub enum OrchestratorMsg {
    StartWorkflow {
        name: String,
        input: Vec<u8>,
        reply: RpcReplyPort<Result<String, StartError>>,
    },
    GetStatus {
        invocation_id: String,
        reply: RpcReplyPort<Option<InstanceStatus>>,
    },
    Signal {
        invocation_id: String,
        signal_name: String,
        payload: Vec<u8>,
        reply: RpcReplyPort<Result<(), SignalError>>,
    },
    ListWorkflows {
        reply: RpcReplyPort<Vec<WorkflowInfo>>,
    },
    Terminate {
        invocation_id: String,
        reply: RpcReplyPort<Result<(), TerminateError>>,
    },
}
```

### WorkflowInstance

```rust
pub struct WorkflowInstance {
    workflow_name: String,
}

pub struct InstanceState {
    invocation_id: String,
    dag: WorkflowGraph,
    journal: Vec<JournalEntry>,
    journal_cursor: u32,
    status: InstanceStatus,
}

#[derive(Message)]
pub enum InstanceMsg {
    ExecuteStep {
        step_id: u32,
        reply: RpcReplyPort<Result<StepOutput, StepError>>,
    },
    Signal {
        signal_name: String,
        payload: Vec<u8>,
        reply: RpcReplyPort<Result<(), SignalError>>,
    },
    GetStatus(RpcReplyPort<Option<InstanceStatus>>),
    GetJournal(RpcReplyPort<Vec<JournalEntry>>),
    Complete { output: Vec<u8> },
    Fail { error: String },
}
```

### Supervision Event Flow

```rust
async fn handle_supervisor_evt(
    &self,
    myself: ActorRef<Self::Msg>,
    event: SupervisionEvent,
    state: &mut Self::State,
) -> Result<(), ActorProcessingErr> {
    match event {
        SupervisionEvent::ActorTerminated(cell, _, reason) => {
            // Remove from registry
            state.instances.retain(|id, (_, ref_)| ref_ != &cell);
            state.running_count = state.running_count.saturating_sub(1);
            tracing::info!("Workflow terminated: {reason:?}");
        }
        SupervisionEvent::ActorFailed(cell, error) => {
            // Remove from registry
            state.instances.retain(|id, (_, ref_)| ref_ != &cell);
            state.running_count = state.running_count.saturating_sub(1);
            tracing::error!("Workflow failed: {error}");
            // Could implement restart policy here
        }
        SupervisionEvent::ActorStarted(cell) => {
            if let Some(name) = cell.get_name() {
                state.running_count += 1;
                tracing::info!("Workflow started: {name}");
            }
        }
        _ => {}
    }
    Ok(())
}
```

### Capacity Enforcement

```rust
async fn handle(
    &self,
    myself: ActorRef<Self::Msg>,
    msg: OrchestratorMsg,
    state: &mut Self::State,
) -> Result<(), ActorProcessingErr> {
    match msg {
        OrchestratorMsg::StartWorkflow { name, input, reply } => {
            // Check capacity
            if state.running_count >= self.max_concurrent {
                reply.send(Err(StartError::AtCapacity {
                    running: state.running_count,
                    max: self.max_concurrent,
                }));
                return Ok(());
            }

            // Spawn workflow instance
            let invocation_id = generate_id();
            let (actor_ref, _) = Actor::spawn_linked(
                Some(format!("{}:{}", name, invocation_id).into()),
                WorkflowInstance::new(name.clone()),
                InstanceConfig { invocation_id: invocation_id.clone(), input, storage: self.storage.clone() },
                myself.clone().into(),
            ).await?;

            state.instances.insert(invocation_id.clone(), (name, actor_ref));
            reply.send(Ok(invocation_id));
        }
        // ...
    }
}
```

## Consequences

### Positive

- Clear separation of concerns
- Capacity enforced at orchestrator level
- Supervision provides fault isolation
- Registry enables signal routing

### Negative

- Two-level message passing (slight latency)
- MasterOrchestrator is a single point of contention

### Future Considerations

- Could add regional supervisors for sharding
- Could add process groups for fan-out parallelism
