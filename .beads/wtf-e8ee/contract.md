# Contract Specification: wtf-frontend Inspector Panel

## Context
- **Feature**: Dynamic form UI for node configuration (InspectorPanel)
- **Bead ID**: wtf-e8ee
- **Bead Title**: wtf-frontend inspector panel
- **Domain Terms**: 
  - Node: A workflow node (FSM/DAG/Procedural) displayed on the canvas
  - InspectorPanel: Right-side panel showing node input/output/config
  - ExecutionState: Current state of a node (Idle, Running, Completed, Failed, etc.)
- **Assumptions**:
  - The inspector_panel.rs already exists but references Oya types
  - The wtf-frontend graph module provides equivalent types
- **Open Questions**: None

## Preconditions
- [ ] P1: InspectorPanel component receives a valid `Node` signal
- [ ] P2: All signals (step_input, step_output, step_error, etc.) contain serializable JSON values
- [ ] P3: The parent component handles the `on_close` callback

## Postconditions
- [ ] Q1: InspectorPanel renders node name and type correctly
- [ ] Q2: InspectorPanel displays execution state badge with correct colors
- [ ] Q3: Input/Output tabs display formatted JSON payloads
- [ ] Q4: Search filter correctly filters JSON lines
- [ ] Q5: Copy button copies current tab content to clipboard
- [ ] Q6: Duration formatting returns correct string for all input cases

## Invariants
- [ ] I1: Panel always displays node metadata when node is selected
- [ ] I2: Tab state persists during session but resets when node changes
- [ ] I3: Error display only renders when execution_state is Failed OR error_text is non-empty

## Error Taxonomy
- **Error::InvalidInput** - Not applicable (UI component, not fallible)
- **Error::NodeNotSelected** - When node signal is None, panel renders nothing
- **Error::JsonSerializationFailed** - When step_input/step_output cannot be formatted

## Contract Signatures
```rust
// Component signature (Dioxus)
pub fn InspectorPanel(
    node: ReadSignal<Option<Node>>,
    step_input: ReadSignal<Option<serde_json::Value>>,
    step_output: ReadSignal<Option<serde_json::Value>>,
    step_error: ReadSignal<Option<String>>,
    step_stack_trace: ReadSignal<Option<String>>,
    step_start_time: ReadSignal<Option<String>>,
    step_end_time: ReadSignal<Option<String>>,
    step_duration_ms: ReadSignal<Option<i64>>,
    step_attempt: ReadSignal<u32>,
    on_close: EventHandler<()>,
) -> Element

// Helper functions
pub const fn status_badge_class(state: ExecutionState) -> &'static str
pub fn format_duration(ms: Option<i64>) -> String
pub fn execution_state_label(state: ExecutionState) -> &'static str
fn should_render_failure(state: ExecutionState, error_text: &str) -> bool
fn filter_lines(text: &str, query: &str) -> String
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Valid Node | Runtime-checked | `Option<Node>` - None renders empty |
| P2: Serializable values | Type system | `ReadSignal<Option<serde_json::Value>>` |
| P3: Close handler exists | Type system | `EventHandler<()>` |

## Violation Examples (REQUIRED)
- VIOLATES Q4: `filter_lines("foo\nbar", "xyz")` -- returns `""` (empty string) for non-matching query
- VIOLATES Q6: `format_duration(Some(999))` -- returns `"999ms"` (correct), not `"1.00s"` (threshold is 1000ms)

## Ownership Contracts
- All parameters are borrowed signals, no ownership transfer
- CopyButton clones text for clipboard API
- No mutation of input signals

## Non-goals
- [ ] Network requests
- [ ] Backend integration
- [ ] State persistence
