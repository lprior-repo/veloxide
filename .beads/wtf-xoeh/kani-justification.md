bead_id: wtf-xoeh
bead_title: CLI command: wtf admin rebuild-views
phase: 5.7
updated_at: 2026-03-22T00:27:00Z

# Kani Justification: wtf admin rebuild-views

## Kani Analysis: NOT APPLICABLE

### Rationale

The current implementation is a **STUB** - the `rebuild_views()` function returns hardcoded zeros without any actual logic:

```rust
async fn rebuild_views(
    _stores: &KvStores,
    _namespace_filter: &Option<String>,
    _view_filter: Option<&ViewName>,
    _show_progress: bool,
) -> Result<RebuildStats, anyhow::Error> {
    Ok(RebuildStats {
        instances_rebuilt: 0,
        timers_rebuilt: 0,
        definitions_rebuilt: 0,
        events_processed: 0,
        duration_ms: 0,
    })
}
```

### Why Kani is Not Applicable

1. **No State Machine**: The stub contains no state machine - just returns a constant
2. **No Critical State Transitions**: There are no state variables that could reach invalid states
3. **No Async State**: The function is async but performs no operations

### Formal Justification for Skipping Kani

| Criterion | Status |
|-----------|--------|
| Critical state machines exist? | NO - stub only |
| State variables that could reach invalid states? | NO |
| What guarantees contract/tests provide | CLI parsing works, dry-run works |
| Reasoning | No panics possible - function returns Ok constant |

### Conclusion
⏭️ **SKIPPED** - Kani analysis deferred until full implementation exists

### When to Revisit
When `rebuild_views()` contains actual:
- JetStream replay logic
- State machine for tracking rebuild progress
- Error handling with state transitions
