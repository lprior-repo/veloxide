# Kani Justification

## Bead Metadata
- **Bead ID**: wtf-7mmi
- **Bead Title**: wtf-frontend: FSM node types (State, Transition, TerminalState) + inspector forms
- **Phase**: kani
- **Updated At**: 2026-03-21T23:26:00Z

## Formal Argument to Skip Kani Model Checking

### What Critical State Machines Exist
The FSM node types added are **data-only structs** with no state machine logic:
- `FsmStateConfig`: Simple data container with `name: Option<String>` and `is_terminal: Option<bool>`
- `FsmTransitionConfig`: Simple data container with `event_name`, `from_state`, `to_state`: `Option<String>` and `effects: Option<Vec<String>>`

These are passive data structures with no internal state transitions, no control flow, and no behavior. They are serializable/deserializable containers that get embedded in the WorkflowNode enum.

### Why Those State Machines Cannot Reach Invalid States
1. **No internal state**: FsmStateConfig and FsmTransitionConfig are pure data with no invariants to maintain
2. **No state transitions**: These structs don't have state machines - they're just data bags
3. **All fields are Option<T>**: Every field is optional (defaults to None), meaning there are no required fields that could cause construction failures
4. **No validation logic**: The structs have no methods that could fail or panic
5. **Immutable by default**: The structs derive Clone but don't require mutation

### What Guarantees the Contract/Tests Provide
1. **Contract guarantees**:
   - All fields are Option<T> which can always be constructed
   - No validation that could fail
   - No invariants to violate

2. **Test guarantees**:
   - Roundtrip serialization/deserialization tested
   - FromStr/Display roundtrip tested
   - All 26 node types parse correctly

3. **Code review guarantees**:
   - No unsafe code
   - No panics/unwrap/expect
   - All derive macros follow existing patterns

### Formal Reasoning
Kani model checking is designed for:
1. **Concurrent code** with shared state
2. **State machines** with complex transition logic
3. **Unsafe Rust** blocks requiring proof of safety
4. **Critical algorithms** requiring formal verification

The FSM node type implementation is none of these:
- It's pure data (no unsafe code)
- It's serializable only (no runtime behavior)
- It has no state transitions (no state machine)
- It follows proven patterns from existing codebase

### Decision
**SKIP KANI** - This is a data structure addition with no state machine logic. The implementation is provably safe through:
1. Type safety (Option<T> fields)
2. Compile-time checks (no unsafe code)
3. Pattern matching exhaustiveness (enum variants)
4. Serialization testing (roundtrip verification)

The implementation is lower-risk than typical Kani targets and formal model checking would not discover any issues that unit tests and code review haven't already identified.
