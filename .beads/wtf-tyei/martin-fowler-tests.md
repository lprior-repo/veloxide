# Martin Fowler Test Plan: POST /api/v1/workflows/validate

bead_id: wtf-tyei
bead_title: wtf-api: POST /api/v1/workflows/validate — workflow definition linting endpoint
phase: test-plan
updated_at: 2026-03-21T23:28:50Z

## Happy Path Tests

### test_returns_valid_true_when_source_has_no_violations
Given: valid Rust workflow source code with no lint violations
When: POST /api/v1/workflows/validate is called
Then: returns 200 with `{valid: true, diagnostics: []}`

### test_returns_valid_true_when_source_has_only_warnings
Given: Rust workflow source containing L004 violation (ctx-in-closure warning)
When: POST /api/v1/workflows/validate is called
Then: returns 200 with `{valid: true, diagnostics: [{code: "WTF-L004", severity: "warning", ...}]}`

### test_returns_valid_false_when_source_has_error_violation
Given: Rust workflow source containing L001 violation (non-deterministic time call)
When: POST /api/v1/workflows/validate is called
Then: returns 200 with `{valid: false, diagnostics: [{code: "WTF-L001", severity: "error", ...}]}`

## Error Path Tests

### test_returns_400_when_body_is_empty_json_object
Given: JSON body `{}` with no source field
When: POST /api/v1/workflows/validate is called
Then: returns 400 with `{"error": "missing_field", "message": "source field required"}`

### test_returns_400_when_source_field_is_wrong_type
Given: JSON body `{"source": 123}`
When: POST /api/v1/workflows/validate is called
Then: returns 400 with `{"error": "invalid_json", ...}`

### test_returns_400_when_source_has_rust_syntax_error
Given: JSON body `{"source": "fn workflow() { let x = unclosed_string;"}`
When: POST /api/v1/workflows/validate is called
Then: returns 400 with `{"error": "parse_error", "message": "..."}`

### test_returns_500_when_linter_crashes
Given: Linter implementation that panics on certain input
When: POST /api/v1/workflows/validate is called
Then: returns 500 (not propagated panic)

## Edge Case Tests

### test_handles_empty_source_string
Given: JSON body `{"source": ""}`
When: POST /api/v1/workflows/validate is called
Then: returns 200 with `{valid: true, diagnostics: []}`

### test_handles_whitespace_only_source
Given: JSON body `{"source": "   \n\n  "}`
When: POST /api/v1/workflows/validate is called
Then: returns 200 with `{valid: true, diagnostics: []}`

### test_returns_multiple_diagnostics_for_multiple_violations
Given: Rust workflow source with L001 + L002 + L003 violations
When: POST /api/v1/workflows/validate is called
Then: returns 200 with `{valid: false, diagnostics: [L001_entry, L002_entry, L003_entry]}`

### test_diagnostic_includes_span_information
Given: Rust source with violation at known byte offset
When: POST /api/v1/workflows/validate is called
Then: diagnostic entry has `span: [start, end]` with correct byte positions

### test_diagnostic_includes_suggestion_when_available
Given: Rust source with a violation that has a known fix
When: POST /api/v1/workflows/validate is called
Then: diagnostic entry has `suggestion: Some("...")`

### test_diagnostic_span_is_none_when_span_unknown
Given: Linter rule that cannot determine span
When: POST /api/v1/workflows/validate is called
Then: diagnostic entry has `span: null`

## Contract Verification Tests

### test_precondition_p1_request_body_has_source_field
Given: Deserialize input
When: field named "source" is missing
Then: returns Error::MissingSourceField

### test_precondition_p2_source_is_string
Given: JSON input
When: "source" field is not a string type
Then: returns Error::InvalidJson

### test_postcondition_q1_response_has_valid_and_diagnostics
Given: Valid request
When: handler returns response
Then: response has `valid: bool` and `diagnostics: Vec<DiagnosticEntry>`

### test_postcondition_q2_valid_true_when_no_errors
Given: Diagnostics contain only warnings
When: response is built
Then: `valid` is `true`

### test_postcondition_q3_diagnostics_contain_all_violations
Given: Source with multiple different violations
When: linter runs
Then: diagnostics contains one entry per violation found

### test_invariant_i1_no_5xx_for_business_errors
Given: Any business error (parse, validation)
When: handler returns error response
Then: status code is 400, not 500

## Given-When-Then Scenarios

### Scenario 1: Valid workflow with no violations
Given: A syntactically valid Rust workflow function with no prohibited calls
When: Client POSTs to /api/v1/workflows/validate
Then:
- HTTP 200 is returned
- Body contains `{valid: true, diagnostics: []}`
- No runtime errors occur

### Scenario 2: Workflow with single error violation
Given: A Rust workflow function that calls `std::time::SystemTime::now()`
When: Client POSTs to /api/v1/workflows/validate
Then:
- HTTP 200 is returned
- `valid` is `false`
- Diagnostics contains one entry with `code: "WTF-L001"`, `severity: "error"`
- Diagnostic includes message explaining the violation
- Diagnostic may include suggestion if available

### Scenario 3: Malformed JSON body
Given: A request body that is not valid JSON
When: Client POSTs to /api/v1/workflows/validate
Then:
- HTTP 400 is returned
- Body contains `{"error": "invalid_json", "message": "..."}`

### Scenario 4: Missing source field
Given: A request body `{"not_source": "..."}`
When: Client POSTs to /api/v1/workflows/validate
Then:
- HTTP 400 is returned
- Body contains `{"error": "missing_field", "message": "source field required"}`

### Scenario 5: Rust syntax parse error
Given: A request body with syntactically invalid Rust code
When: Client POSTs to /api/v1/workflows/validate
Then:
- HTTP 400 is returned
- Body contains `{"error": "parse_error", "message": "..."}`
- Message describes the parse failure
