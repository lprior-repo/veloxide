# Martin Fowler Test Plan: wtf-frontend Inspector Panel

## Happy Path Tests
- `test_status_badge_class_returns_correct_css_for_running_state`
  - Given: ExecutionState::Running
  - When: status_badge_class is called
  - Then: returns string containing "blue"

- `test_status_badge_class_returns_correct_css_for_completed_state`
  - Given: ExecutionState::Completed
  - When: status_badge_class is called
  - Then: returns string containing "green"

- `test_format_duration_returns_ms_for_subsecond_values`
  - Given: Some(234)
  - When: format_duration is called
  - Then: returns "234ms"

- `test_format_duration_returns_seconds_for_exactly_one_second`
  - Given: Some(1000)
  - When: format_duration is called
  - Then: returns "1.00s"

- `test_execution_state_label_returns_pending_for_queued`
  - Given: ExecutionState::Queued
  - When: execution_state_label is called
  - Then: returns "pending"

## Error Path Tests
- `test_format_duration_returns_dash_for_none`
  - Given: None
  - When: format_duration is called
  - Then: returns "—"

- `test_filter_lines_returns_empty_for_non_matching_query`
  - Given: text="foo\nbar", query="xyz"
  - When: filter_lines is called
  - Then: returns ""

## Edge Case Tests
- `test_filter_lines_returns_all_lines_for_empty_query`
  - Given: text="foo\nbar\nbaz", query=""
  - When: filter_lines is called
  - Then: returns "foo\nbar\nbaz"

- `test_filter_lines_is_case_insensitive`
  - Given: text="FooBar\nfoobar", query="foobar"
  - When: filter_lines is called
  - Then: returns both lines containing "foobar" (case-insensitive)

- `test_format_duration_handles_zero`
  - Given: Some(0)
  - When: format_duration is called
  - Then: returns "0ms"

- `test_format_duration_handles_large_values`
  - Given: Some(60000)
  - When: format_duration is called
  - Then: returns "60.00s"

## Contract Verification Tests
- `test_precondition_p3_close_handler_is_optional` (implicit in Dioxus)
- `test_postcondition_q1_node_name_display`
- `test_postcondition_q2_execution_badge_colors`

## Given-When-Then Scenarios

### Scenario 1: Display Running Node
Given: A node with execution_state=Running and name="ProcessOrder"
When: InspectorPanel renders
Then:
- Node name "ProcessOrder" is displayed
- Badge shows "running" with blue background
- Start/End/Duration/Attempt fields are populated

### Scenario 2: Search Filters Output
Given: Output JSON with multiple lines and user searches for "error"
When: filter_lines is called with query="error"
Then: Only lines containing "error" (case-insensitive) are returned

### Scenario 3: Copy Button Copies Tab Content
Given: User is on Output tab with JSON content
When: Copy button is clicked
Then: Current tab's content is copied to clipboard

### Scenario 4: Failed State Shows Error Details
Given: execution_state=Failed and error_text="Connection refused"
When: InspectorPanel renders Output tab
Then: Error panel displays with red styling showing "Connection refused"
