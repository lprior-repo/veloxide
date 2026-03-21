# ADR-004: Step Functions Parity for State Types

## Status

Accepted

## Context

wtf-engine aims for **AWS Step Functions feature parity** with the following constraints:

- **Max wait time: 24 hours** (no year-long waits)
- **Single machine** (no distributed AWS integrations)
- **Rust-native activities** (no Lambda, no AWS SDK)

### Step Functions State Types

| Category | States |
|----------|--------|
| **Flow Control** | `Pass`, `Choice`, `Parallel`, `Map`, `Succeed`, `Fail` |
| **Execution** | `Task` (invoke activity), `Wait` (delay) |
| **Service Integration** | 50+ AWS integrations (not applicable - we do Rust closures) |

### Required Parity

| State Type | Required | Notes |
|------------|----------|-------|
| `Pass` | ✅ | result + result_path |
| `Task` | ✅ | resource, timeout, heartbeat, retry, catch |
| `Wait` | ✅ | seconds (max 24h), timestamp (max 24h from now) |
| `Choice` | ✅ | all comparison operators |
| `Parallel` | ✅ | branches, retry, catch |
| `Map` | ✅ | iterator, max_concurrency, retry, catch |
| `Succeed` | ✅ | output |
| `Fail` | ✅ | error, cause |
| `Retry` | ✅ | ErrorEquals, IntervalSeconds, MaxAttempts, BackoffRate |
| `Catch` | ✅ | ErrorEquals, Next, ResultPath |

## Decision

We will implement **full Step Functions parity** for state types, minus long waits.

### StateConfig Enum

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type")]
pub enum StateConfig {
    // Pass - No-op passthrough
    Pass {
        result: Option<serde_json::Value>,
        result_path: Option<String>,
    },

    // Task - Activity invocation
    Task {
        resource: String,           // Activity name
        timeout: Option<DurationSecs>,
        heartbeat: Option<DurationSecs>,
        retry: Option<Vec<RetryPolicy>>,
        catch: Option<Vec<CatchPolicy>>,
    },

    // Wait - Delay (max 24 hours)
    Wait {
        seconds: Option<u64>,        // Relative wait (max 86400)
        timestamp: Option<String>,  // ISO8601 absolute (max 24h from now)
    },

    // Choice - Branching
    Choice {
        choice: Vec<ChoiceRule>,
        default: Option<String>,
    },

    // Parallel - Fan-out/fan-in
    Parallel {
        branches: Vec<Branch>,
        retry: Option<Vec<RetryPolicy>>,
        catch: Option<Vec<CatchPolicy>>,
    },

    // Map - Iterate over list
    Map {
        iterator: Box<StateConfig>,
        items_path: String,
        max_concurrency: Option<u32>,
        retry: Option<Vec<RetryPolicy>>,
        catch: Option<Vec<CatchPolicy>>,
    },

    // Terminal states
    Succeed {
        output: Option<serde_json::Value>,
    },

    Fail {
        error: String,
        cause: Option<String>,
    },
}
```

### Error Handling

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub errors: Vec<String>,        // Error names to retry
    pub interval_sec: u64,
    pub max_attempts: u32,
    pub backoff_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatchPolicy {
    pub errors: Vec<String>,
    pub next: String,
    pub result_path: Option<String>,
}
```

### Wait State Constraints

```rust
impl WaitConfig {
    const MAX_WAIT_SECONDS: u64 = 24 * 60 * 60; // 24 hours

    pub fn validate(&self) -> Result<(), ValidationError> {
        if let Some(seconds) = self.seconds {
            if seconds > Self::MAX_WAIT_SECONDS {
                return Err(ValidationError::WaitExceeds24Hours);
            }
        }
        if let Some(ts) = &self.timestamp {
            // Parse ISO8601, validate <= 24h from now
        }
        Ok(())
    }
}
```

## Consequences

### Positive

- Familiar API for AWS Step Functions users
- Comprehensive error handling (retry/catch)
- Rich control flow (choice, parallel, map)
- Proven patterns from AWS

### Negative

- Complex state types (many fields)
- More code to implement correctly

### Excluded Features

- Long waits (> 24 hours) - Not applicable for single-machine
- AWS service integrations - We invoke Rust closures, not AWS services
- Express vs Standard workflows - Single mode only
