# Contract Specification: POST /api/v1/workflows/validate

bead_id: wtf-tyei
bead_title: wtf-api: POST /api/v1/workflows/validate — workflow definition linting endpoint
phase: contract
updated_at: 2026-03-21T23:28:50Z

## Context

- **Feature**: Lint workflow Rust source code via HTTP API
- **Domain terms**:
  - `LintCode` (L001-L006): Rule identifiers per ADR-020
  - `Severity`: Error | Warning
  - `Diagnostic`: { code, severity, message, suggestion, span }
  - `Workflow source`: Rust source code as a string
- **Assumptions**:
  - wtf-linter crate provides lint rules L001-L006
  - Parse errors in source code return 400 with descriptive message
  - Empty source returns valid=true with empty diagnostics
- **Open questions**: None

## Preconditions

1. **P1**: Request body must be valid JSON object with `source` field containing string
2. **P2**: `source` field must be non-null (can be empty string)

## Postconditions

1. **Q1**: Response is JSON object with `valid: bool` and `diagnostics: Vec<DiagnosticEntry>`
2. **Q2**: `valid` is `true` iff `diagnostics` is empty OR contains only warnings (no errors)
3. **Q3**: `diagnostics` contains entry for each L001-L006 violation found
4. **Q4**: Each diagnostic entry has: `code` (string), `severity` (string), `message` (string), `suggestion` (Option<string>), `span` (Option<[start, end]>)
5. **Q5**: HTTP 400 returned when source cannot be parsed as Rust syntax

## Invariants

1. **I1**: Response status code is 200, 400, or 500 (no 5xx for business logic errors)

## Error Taxonomy

- `LintError::ParseError(String)` → HTTP 400 with JSON `{"error": "parse_error", "message": <details>}`
- `Error::InvalidJson` → HTTP 400 with `{"error": "invalid_json", "message": "..."}`
- `Error::MissingSourceField` → HTTP 400 with `{"error": "missing_field", "message": "source field required"}`

## Contract Signatures

```rust
pub async fn validate_workflow(
    Json(req): Json<ValidateWorkflowRequest>
) -> impl IntoResponse

#[derive(Deserialize)]
struct ValidateWorkflowRequest {
    source: String,  // Rust workflow source code
}

#[derive(Serialize)]
struct ValidateWorkflowResponse {
    valid: bool,
    diagnostics: Vec<DiagnosticEntry>,
}

struct DiagnosticEntry {
    code: String,           // "WTF-L001" etc.
    severity: String,       // "error" | "warning"
    message: String,
    suggestion: Option<String>,
    span: Option<[usize; 2]>,
}
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Valid JSON body | Runtime-checked | `serde_json::from_reader` → 400 on fail |
| P2: source is string | Compile-time | `Json<ValidateWorkflowRequest>` with `#[derive(Deserialize)]` |
| Q1-Q5: Response shape | Compile-time | `#[derive(Serialize)]` structs |

## Violation Examples (REQUIRED)

- VIOLATES P1: `{}` (no source field) → `Err(Error::MissingSourceField)` → 400
- VIOLATES P1: `{"source": 123}` (wrong type) → `Err(Error::InvalidJson)` → 400
- VIOLATES Q5: `{"source": "fn workflow() { let x = unclosed_string;"}` → `Err(LintError::ParseError(...))` → 400
- VIOLATES Q2: source with L001 error → `valid: false`
- VIOLATES Q3: source with L004 warning only → `valid: true`

## Ownership Contracts

- No ownership transfer (read-only linting)
- `source` string is borrowed, not cloned in success path
- Diagnostic entries are cloned into response

## Non-goals

- Actually executing/compiling the workflow code
- Type checking beyond syntax parse
- Returning fix suggestions for all rules (only where implemented)
