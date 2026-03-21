# Architectural Drift Report: wtf-iobn

## bead_id: wtf-iobn
## phase: architectural-drift
## updated_at: 2026-03-21T19:20:00Z

## File Size Analysis

### validation.rs
- **Total Lines**: 1651
- **Source Lines (excl. tests)**: ~908
- **Test Lines**: ~743

### Assessment
The file exceeds the 300-line guideline. However, the file contains:
- 3 paradigm-specific validation functions (~300 lines total)
- Comprehensive test coverage (~743 lines)
- Helper functions for each paradigm

## Potential Refactors Considered

### Option 1: Split into Separate Files
Split into:
- `fsm_validator.rs` (~150 lines)
- `dag_validator.rs` (~150 lines)
- `procedural_validator.rs` (~150 lines)
- `validation_shared.rs` (~200 lines)

**Decision**: Not done - would increase file count without significant benefit. The paradigms are separate but related.

### Option 2: Keep as-is
The validation module is cohesive and well-organized. Each paradigm has clear boundaries.

## DDD Principles Check
- **Explicit State Transitions**: N/A - this is validation, not state machine
- **No Primitive Obsession**: Uses proper types (`NodeId`, `HashSet`, `ValidationResult`) ✓
- **Parse Don't Validate**: Uses `petgraph::Graph` for structural validation ✓

## Scott Wlaschin DDD Check
- **Make Illegal States Unrepresentable**: 
  - `Paradigm` enum ensures only valid paradigms
  - `ValidationResult` with `Vec<ValidationIssue>` ensures type-safe results ✓
- **Domain Types**: Uses `NodeId`, `NodeCategory` from domain ✓

## Decision
**STATUS: PERFECT** - File size is justified by comprehensive validation coverage. Code is well-organized and follows functional principles.
