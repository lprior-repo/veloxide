bead_id: wtf-xoeh
bead_title: CLI command: wtf admin rebuild-views
phase: 2
updated_at: 2026-03-21T23:59:00Z

# Test Plan Defects

## STATUS: REJECTED

Review against Testing Trophy, Dan North BDD, and Dave Farley ATDD doctrines found the following defects:

---

## Critical Defects

### DEFECT-1: Missing test for VIOLATES Q1 (Data Loss Bug)
**Contract violation**: Rebuild completes but some instances missing from `wtf-instances` → Data loss bug
**Required test**: `test_no_data_loss_postcondition`
- Given: JetStream has N instances
- When: Rebuild completes successfully
- Then: Exactly N entries exist in `wtf-instances`
- And: Every instance_id from JetStream has a corresponding KV entry

### DEFECT-2: Missing test for VIOLATES Q2 (Exit Code Inconsistency)
**Contract violation**: `rebuild_views()` returns success but exit code is non-zero → Inconsistent signaling
**Required test**: `test_exit_code_matches_return_value`
- Given: A rebuild scenario
- When: Command completes
- Then: Exit code is 0 if and only if return value is Ok

---

## Major Defects

### DEFECT-3: Missing test for --dry-run flag
**Contract specifies**: `--dry-run` flag should print what would be rebuilt without rebuilding
**Required test**: `test_dry_run_does_not_modify_state`
- Given: KV buckets have existing data
- When: I run `wtf admin rebuild-views --dry-run`
- Then: Exit code is 0
- And: KV buckets are unchanged (verified by reading before/after)

### DEFECT-4: Missing test for concurrent rebuild attempts
**Invariant**: No other writer modifying KV during rebuild
**Required test**: `test_concurrent_rebuild_returns_error_or_is_serialized`
- Given: A rebuild is in progress
- When: A second rebuild is started concurrently
- Then: Either second rebuild fails with error, or rebuilds are properly serialized
- And: No corrupted state in KV buckets

---

## Minor Defects

### DEFECT-5: Ambiguous heartbeat test
**Current**: `test_heartbeat_bucket_not_rebuilt` is vague
**Required**: Explicit Given-When-Then
- Given: `wtf-heartbeats` bucket has entries
- When: I run `wtf admin rebuild-views`
- Then: `wtf-heartbeats` entries remain unchanged (heartbeats are NOT replayed)
- And: Other three buckets are rebuilt

---

## Summary

| Defect | Severity | Fix Required |
|--------|----------|--------------|
| DEFECT-1 | Critical | Add test_no_data_loss_postcondition |
| DEFECT-2 | Critical | Add test_exit_code_matches_return_value |
| DEFECT-3 | Major | Add test_dry_run_does_not_modify_state |
| DEFECT-4 | Major | Add test_concurrent_rebuild_returns_error_or_is_serialized |
| DEFECT-5 | Minor | Clarify test_heartbeat_bucket_not_rebuilt |

**Max retries remaining**: 3
