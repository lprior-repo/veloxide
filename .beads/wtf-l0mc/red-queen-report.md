# Red Queen Report: wtf-l0mc — Foundation Integration Tests

## Bead Overview
**Bead ID:** wtf-l0mc  
**Title:** Foundation integration tests (wtf-common + wtf-storage)  
**Phase:** State 5 (Adversarial Review)

## Red Queen Testing Strategy

Red Queen testing attempts to break the implementation through adversarial inputs and edge cases.

## Test Cases Attempted

### Edge Cases on ID Validation

| Test Case | Input | Expected | Actual | Status |
|-----------|-------|----------|--------|--------|
| InstanceId with dot | "01ARZ.BAD" | Err(InvalidNatsId) | Err(InvalidNatsId) | ✓ PASS |
| InstanceId with star | "01ARZ*" | Err(InvalidNatsId) | Err(InvalidNatsId) | ✓ PASS |
| InstanceId with GT | "01ARZ>" | Err(InvalidNatsId) | Err(InvalidNatsId) | ✓ PASS |
| NamespaceId with dot | "pay.ments" | Err(InvalidNatsId) | Err(InvalidNatsId) | ✓ PASS |
| NamespaceId with space | "pay ments" | Err(InvalidNatsId) | Err(InvalidNatsId) | ✓ PASS |

### Serialization Edge Cases

| Test Case | Input | Expected | Actual | Status |
|-----------|-------|----------|--------|--------|
| Empty bytes | WorkflowEvent::InstanceStarted | Deserialize fails | Correct behavior | ✓ PASS |
| Corrupted checksum | SnapshotRecord | is_valid() = false | is_valid() = false | ✓ PASS |
| Missing snapshot | Non-existent InstanceId | Ok(None) | Ok(None) | ✓ PASS |

### Replay Sequence Calculation

| Test Case | Input | Expected | Actual | Status |
|-----------|-------|----------|--------|--------|
| With snapshot | Some(100) | 101 | 101 | ✓ PASS |
| Without snapshot | None | 1 | 1 | ✓ PASS |

## Adversarial Findings

### No Critical Issues Found

The implementation correctly handles:
- Invalid NATS subject characters (`.`, `>`, `*`, whitespace)
- Checksum validation for snapshots
- Missing key handling (returns None, not error)
- Replay sequence calculation

### Potential Improvement Areas

1. **NATS Connection Retry**: The retry logic (500ms, 1s, 2s) is hardcoded. Consider making delays configurable.
2. **Snapshot Corruption Detection**: Currently returns None on corruption. Consider adding a metric/counter for observability.

## Defects Found

**None** — All adversarial tests pass as expected.

## Red Queen Gate Decision

**Status:** APPROVED

All Red Queen test cases pass. No defects found that would cause incorrect behavior.
