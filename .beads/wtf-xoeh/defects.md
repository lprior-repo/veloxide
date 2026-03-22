bead_id: wtf-xoeh
bead_title: CLI command: wtf admin rebuild-views
phase: 5.5
updated_at: 2026-03-22T00:25:00Z

# Black Hat Review: wtf admin rebuild-views

## Status: REVIEWABLE (Stub Implementation)

### Code Reviewed
- `crates/wtf-cli/src/main.rs` - CLI entry point
- `crates/wtf-cli/src/admin.rs` - rebuild-views command
- `crates/wtf-cli/src/lib.rs` - module exports

### Findings

#### ✅ Structural Quality
1. Proper module organization
2. Async/await used correctly
3. Error handling via anyhow::Result
4. Clap derive for CLI parsing
5. No unsafe code
6. No panics (enforced by deny attributes)

#### ⚠️ Needs Improvement
1. **STUB**: `rebuild_views()` returns hardcoded zeros - not implemented
2. Error taxonomy incomplete - `WtfError` variants not aligned with contract
3. Missing proper exit codes for different error types

#### ❌ Missing (Not Reviewable Until Implemented)
1. Actual JetStream replay logic
2. KV bucket update logic
3. Progress reporting implementation
4. Namespace filtering logic
5. View-specific rebuild logic

## Defects Found
| ID | Severity | Description |
|----|----------|-------------|
| DEFECT-BH-1 | HIGH | `rebuild_views()` is a stub - returns zeros without doing any work |
| DEFECT-BH-2 | MEDIUM | Error variants in `WtfError` don't match contract (no NatsConnect, StreamNotFound, BucketNotFound) |

## Black Hat Verdict
**STATUS: REJECTED** - Implementation is incomplete (stub only)

## Required Actions
1. Implement actual rebuild_views() logic
2. Add error variants to WtfError or use custom error type
3. Test with live NATS before approval
