# ADR-012: API Design (HTTP/REST)

## Status

Accepted

## Context

wtf-engine needs an **API** for:

1. **Frontend** - Workflow editor, execution monitoring
2. **CLI** - Human interaction
3. **External systems** - Integration with other tools
4. **AI offloading** - Machine-readable interface

### Design Principles

1. **REST** - Resource-oriented, predictable URLs
2. **JSON** - Human-readable, widely supported
3. **Versioned** - `/api/v1/` prefix for future compatibility
4. **Documented** - OpenAPI/Swagger spec

## Decision

We will implement an **Axum-based HTTP API** with REST conventions.

### Base URL Structure

```
http://localhost:8080/
├── health                           # Health check
├── api/v1/
│   ├── workflows                    # Workflow management
│   │   ├── POST   /                # Start workflow
│   │   ├── GET    /                # List workflows
│   │   ├── GET    /:id             # Get workflow status
│   │   ├── DELETE /:id             # Terminate workflow
│   │   ├── POST   /:id/signals    # Send signal
│   │   └── GET    /:id/journal    # Get journal
│   │
│   ├── definitions                 # Workflow definitions
│   │   ├── GET    /                # List definitions
│   │   ├── POST   /                # Create definition
│   │   ├── GET    /:name           # Get definition
│   │   └── DELETE /:name           # Delete definition
│   │
│   ├── activities                  # Activity registry
│   │   ├── GET    /                # List activities
│   │   └── POST   /                # Register activity
│   │
│   ├── runs                        # Execution history
│   │   ├── GET    /                # List runs
│   │   └── GET    /:id             # Get run details
│   │
│   └── stats                       # Statistics
│       └── GET    /                # Get system stats
```

### Request/Response Examples

#### Start Workflow

```http
POST /api/v1/workflows
Content-Type: application/json

{
    "workflow_name": "checkout",
    "input": {
        "order_id": "ord_123",
        "total": 99.99
    }
}
```

```http
HTTP/1.1 201 Created
Content-Type: application/json

{
    "invocation_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "workflow_name": "checkout",
    "status": "running",
    "started_at": "2024-01-15T10:30:00Z"
}
```

#### Get Workflow Status

```http
GET /api/v1/workflows/01ARZ3NDEKTSV4RRFFQ69G5FAV
```

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
    "invocation_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "workflow_name": "checkout",
    "status": "running",
    "current_step": 3,
    "started_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:31:00Z"
}
```

#### Send Signal

```http
POST /api/v1/workflows/01ARZ3NDEKTSV4RRFFQ69G5FAV/signals
Content-Type: application/json

{
    "signal_name": "payment_approved",
    "payload": {
        "approved": true,
        "transaction_id": "txn_789"
    }
}
```

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
    "acknowledged": true
}
```

#### Get Journal

```http
GET /api/v1/workflows/01ARZ3NDEKTSV4RRFFQ69G5FAV/journal
```

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
    "invocation_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "entries": [
        {
            "seq": 0,
            "type": "Run",
            "name": "validate_order",
            "input": {"order_id": "ord_123"},
            "output": {"valid": true},
            "timestamp": "2024-01-15T10:30:01Z"
        },
        {
            "seq": 1,
            "type": "Run",
            "name": "charge_card",
            "input": {"total": 99.99},
            "output": {"receipt_id": "rcpt_456"},
            "timestamp": "2024-01-15T10:30:05Z"
        },
        {
            "seq": 2,
            "type": "Wait",
            "duration_ms": 86400000,
            "fire_at": "2024-01-16T10:30:05Z",
            "status": "waiting"
        }
    ]
}
```

#### Error Response

```http
HTTP/1.1 409 Conflict
Content-Type: application/json

{
    "error": "at_capacity",
    "message": "3 workflows running (max 3)",
    "retry_after_seconds": 5
}
```

### Implementation (Axum)

```rust
// crates/wtf-api/src/handlers.rs

pub async fn start_workflow(
    Json(payload): Json<StartWorkflowRequest>,
) -> Result<Json<StartWorkflowResponse>, StatusCode> {
    let invocation_id = master_orchestrator
        .start_workflow(&payload.workflow_name, &payload.input)
        .await
        .map_err(|e| map_error(e))?;

    Ok(Json(StartWorkflowResponse {
        invocation_id,
        workflow_name: payload.workflow_name,
        status: "running".to_string(),
        started_at: Utc::now(),
    }))
}

pub async fn get_workflow(
    Path(invocation_id): Path<String>,
) -> Result<Json<WorkflowStatus>, StatusCode> {
    let status = master_orchestrator
        .get_status(&invocation_id)
        .await
        .map_err(|e| map_error(e))?;

    match status {
        Some(s) => Ok(Json(s)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn send_signal(
    Path(invocation_id): Path<String>,
    Json(payload): Json<SignalRequest>,
) -> Result<Json<SignalResponse>, StatusCode> {
    master_orchestrator
        .signal(&invocation_id, &payload.signal_name, &payload.payload)
        .await
        .map_err(|e| map_error(e))?;

    Ok(Json(SignalResponse { acknowledged: true }))
}
```

### API Versioning Strategy

- **Current**: `/api/v1/`
- **Future**: `/api/v2/` when breaking changes needed
- **Deprecation**: Old versions supported for 6 months after new version

## Consequences

### Positive

- **Familiar** - REST is widely understood
- **Documentable** - OpenAPI spec generation
- **Debuggable** - Human-readable JSON
- **Tooling** - curl, Postman, etc.

### Negative

- **Verbose** - HTTP overhead for each request
- **Not real-time** - No WebSocket for live updates (future enhancement)

### Future Considerations

- **WebSocket** - For real-time execution updates
- **GraphQL** - For flexible querying (if needed)
- **gRPC** - For internal communication (if distributed)
