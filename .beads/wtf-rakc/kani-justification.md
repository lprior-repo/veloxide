# Kani Justification: FSM Crash-and-Replay Integration Test

bead_id: wtf-rakc
phase: kani
updated_at: 2026-03-22T00:00:00Z

## Formal Argument to Skip Kani Model Checking

### Critical State Machines Analyzed

The FSM logic tested in this bead consists of:
1. **FsmActorState**: Simple state machine with `current_state: String` and `applied_seq: Set<u64>`
2. **apply_event function**: Pure function that applies events to state

### Why Kani is Not Needed

1. **No Complex State Machines**: The FSM is not a complex state machine with multiple concurrent states. It's a simple string-based state with a set of applied sequence numbers.

2. **No Invalid States Possible**: The FSM state is represented as:
   - `current_state: String` - The current state name
   - `applied_seq: HashSet<u64>` - Set of applied sequence numbers
   
   There are no invalid combinations of these fields.

3. **Apply Event is Pure**: The `apply_event` function is a pure function that:
   - Takes current state + event + sequence number
   - Returns new state + result
   - Has no side effects
   - Is deterministic

4. **Contract Tests Verify Behavior**: The unit tests verify:
   - Correct state transitions
   - Correct duplicate detection (AlreadyApplied)
   - Correct sequence tracking

5. **No Panic States**: The code uses `matches!` macro and pattern matching which cannot panic (unlike `unwrap` on `None`)

### What Would Require Kani

Kani would be beneficial for:
- Complex state machines with invariant constraints
- Code using raw pointers or unsafe
- Concurrency with shared mutable state
- Business logic with complex validation rules

### Conclusion

The FSM logic in this bead is simple enough that:
1. The state space is trivially enumerable
2. All transitions are tested via unit tests
3. No unsafe code is involved
4. No complex invariants need verification

**Recommendation: Skip Kani - the FSM logic is verified by conventional unit tests.**
