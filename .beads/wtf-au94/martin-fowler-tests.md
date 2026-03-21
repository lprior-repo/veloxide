# Martin Fowler Tests: wtf-au94 — Phase 1 Foundation

## Test Strategy
Given-When-Then format for behavior-driven verification of foundation components.

---

## Feature: WorkflowEvent Serialization

### Scenario: Event roundtrip through msgpack
**Given** a WorkflowEvent instance (e.g., ActivityCompleted)  
**When** I serialize it with `to_msgpack()` and deserialize with `from_msgpack()`  
**Then** the deserialized event equals the original  

```
Given a WorkflowEvent instance
  And it has fields: activity_id="act-001", result=Bytes("ok"), duration_ms=42
When serializing to msgpack
  And deserializing from msgpack
Then the result equals the original event
```

### Scenario: JSON debug representation uses snake_case tags
**Given** a WorkflowEvent instance  
**When** I serialize it with `serde_json::to_string()`  
**Then** the JSON contains `"type":"snake_case_variant_name"`  

---

## Feature: NATS Connection Management

### Scenario: Connect retries with exponential backoff
**Given** NatsConfig with urls=["nats://localhost:4222"]  
**And** embedded=false  
**When** I call `connect(&config)` but NATS is unavailable  
**Then** it retries 3 times with delays [500ms, 1s, 2s]  
**And** returns WtfError::NatsPublish on final failure  

### Scenario: Embedded server starts automatically
**Given** NatsConfig with embedded=true  
**When** I call `connect(&config)`  
**Then** it spawns nats-server subprocess on port 4222  
**And** connects to it  

---

## Feature: JetStream Event Append

### Scenario: append_event awaits PublishAck before returning
**Given** a connected JetStream context  
**And** a NamespaceId and InstanceId  
**When** I call `append_event(js, namespace, instance_id, event)`  
**Then** it publishes to subject `wtf.log.<namespace>.<instance_id>`  
**And** it awaits PublishAck within 5 seconds  
**And** returns Ok(sequence_number)  

### Scenario: append_event fails on publish error
**Given** a connected JetStream context  
**And** invalid subject characters  
**When** I call `append_event()`  
**Then** it returns WtfError::NatsPublish  

---

## Feature: Stream Provisioning

### Scenario: provision_streams is idempotent
**Given** JetStream context  
**When** I call `provision_streams()` twice  
**Then** both calls succeed (stream already exists = OK)  
**And** streams wtf-events, wtf-work, wtf-signals, wtf-archive exist  

### Scenario: verify_streams passes when all streams exist
**Given** all four streams are provisioned  
**When** I call `verify_streams()`  
**Then** it returns Ok(())  

### Scenario: verify_streams fails when stream missing
**Given** a stream is deleted  
**When** I call `verify_streams()`  
**Then** it returns WtfError::NatsPublish  

---

## Feature: KV Bucket Operations

### Scenario: heartbeat key format
**Given** an InstanceId "01ARZ"  
**When** I call `heartbeat_key(&instance_id)`  
**Then** it returns "hb/01ARZ"  

### Scenario: write_heartbeat creates entry with TTL
**Given** a provisioned `wtf-heartbeats` KV store  
**When** I call `write_heartbeat(store, instance_id, node_id)`  
**Then** the entry is created with key `hb/<instance_id>`  
**And** TTL is 10 seconds  

### Scenario: delete_heartbeat removes entry
**Given** a heartbeat entry exists  
**When** I call `delete_heartbeat(store, instance_id)`  
**Then** the entry is deleted  

---

## Feature: Snapshot Storage

### Scenario: SnapshotRecord validates checksum
**Given** a SnapshotRecord with state_bytes and computed checksum  
**When** I call `is_valid()`  
**Then** it returns true if checksum matches  
**And** false if checksum is corrupted  

### Scenario: write_snapshot stores to sled
**Given** an open sled database  
**And** an InstanceId and SnapshotRecord  
**When** I call `write_snapshot(db, instance_id, record)`  
**Then** the record is stored under the `snapshots` tree  
**And** subsequent `read_snapshot` returns the same record  

### Scenario: read_snapshot returns None for missing key
**Given** an open sled database  
**And** no snapshot for instance_id  
**When** I call `read_snapshot(db, instance_id)`  
**Then** it returns Ok(None)  

### Scenario: Corrupted snapshot returns None
**Given** a snapshot with wrong checksum is stored  
**When** I call `read_snapshot(db, instance_id)`  
**Then** it returns Ok(None) (corruption detected, fallback to replay)  

---

## Feature: Replay Consumer

### Scenario: replay_start_seq returns snapshot_seq + 1
**Given** snapshot_seq = Some(100)  
**When** I call `replay_start_seq(snapshot_seq)`  
**Then** it returns 101  

### Scenario: replay_start_seq returns 1 when no snapshot
**Given** snapshot_seq = None  
**When** I call `replay_start_seq(None)`  
**Then** it returns 1 (full replay)  

---

## Feature: ID Validation

### Scenario: InstanceId rejects NATS-illegal characters
**Given** a string with dots, stars, or gt signs  
**When** I call `InstanceId::try_new(string)`  
**Then** it returns Err(InvalidNatsId)  

### Scenario: NamespaceId validates NATS subject safety
**Given** a namespace string with whitespace  
**When** I call `NamespaceId::try_new(string)`  
**Then** it returns Err(InvalidNatsId)  

---

## Integration Test Scenarios

### Scenario: Full workflow instance lifecycle
**Given** NATS connected and streams provisioned  
**When** I start a new instance  
**And** dispatch an activity  
**And** the activity completes  
**And** I take a snapshot  
**Then** I can recover from the snapshot  
**And** replay from seq+1 produces the same state  

### Scenario: Heartbeat failure triggers recovery
**Given** an active workflow instance with heartbeat  
**When** the heartbeat expires (10s TTL)  
**Then** the instance is detected as crashed  
**And** recovery is triggered  

---

## Child Beads

| Bead | Test Type | Coverage |
|------|-----------|----------|
| wtf-au95 | Integration Tests | append_event, provision_streams, provision_kv_buckets, replay |
| wtf-au96 | Chaos Tests | Network partitions, NATS crashes, corruption |
