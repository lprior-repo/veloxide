# Martin Fowler Test Plan: WTF-L005 tokio-spawn-in-workflow

## Happy Path Tests
- `test_lints_clean_code_with_no_violations` - Given valid Rust code with no tokio::spawn, when linted, then returns empty diagnostics
- `test_lints_multiple_workflow_functions_in_same_file` - Given file with multiple impl WorkflowFn, when linted, then all are checked

## Error Path Tests
- `test_returns_parse_error_for_invalid_rust` - Given invalid Rust syntax, when linted, then returns LintError::ParseError
- `test_returns_empty_for_non_workflow_code` - Given Rust code without WorkflowFn impl, when linted, then returns empty diagnostics

## Edge Case Tests
- `test_handles_empty_source` - Given empty string, when linted, then returns empty diagnostics
- `test_handles_tokio_spawn_in_nested_block` - Given tokio::spawn inside if/while/loop, when linted, then detects it
- `test_handles_tokio_spawn_in_match_arm` - Given tokio::spawn inside match arm, when linted, then detects it
- `test_handles_tokio_spawn_in_closure` - Given tokio::spawn inside a closure within workflow, when linted, then detects it
- `test_handles_multiple_tokio_spawns` - Given multiple tokio::spawn calls, when linted, then returns all violations

## Contract Verification Tests
- `test_precondition_valid_rust_source` - Verify parse error handling
- `test_postcondition_all_violations_found` - Verify no false negatives
- `test_invariant_no_panic_on_any_ast` - Verify graceful error handling

## Contract Violation Tests

### test_violation_tokio_spawn_in_workflow_basic
```rust
#[derive(Debug)]
struct MyWorkflow;
#[async_trait::async_trait]
impl WorkflowFn for MyWorkflow {
    async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> {
        tokio::spawn(async {
            println!("detached task");
        });
        Ok(())
    }
}
```
Given: tokio::spawn inside impl WorkflowFn::execute
When: lint_workflow_code() is called
Then: returns Diagnostic { code: L005, severity: Error, message: contains "tokio::spawn" }

### test_violation_nested_tokio_spawn_in_workflow
```rust
impl WorkflowFn for MyWorkflow {
    async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> {
        let _ = if condition {
            tokio::spawn(async {
                do_work().await;
            })
        } else {
            Ok(())
        };
        Ok(())
    }
}
```
Given: tokio::spawn nested inside if expression in workflow fn
When: lint_workflow_code() is called
Then: returns Diagnostic { code: L005, span: points to tokio::spawn }

### test_violation_tokio_spawn_in_closure_within_workflow
```rust
impl WorkflowFn for MyWorkflow {
    async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> {
        let numbers = vec![1, 2, 3];
        numbers.iter().for_each(|_| {
            tokio::spawn(async {  // violation here
                println!("in closure");
            });
        });
        Ok(())
    }
}
```
Given: tokio::spawn inside for_each closure within workflow fn
When: lint_workflow_code() is called
Then: returns Diagnostic { code: L005 }

### test_no_false_positive_outside_workflow
```rust
async fn helper_function() {
    tokio::spawn(async {
        println!("helper task");
    });
}
```
Given: tokio::spawn inside a regular async function (not WorkflowFn)
When: lint_workflow_code() is called
Then: returns empty diagnostics (no false positive)

### test_no_false_positive_different_spawn
```rust
impl WorkflowFn for MyWorkflow {
    async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> {
        let handle = std::thread::spawn(|| {
            println!("thread");
        });
        Ok(())
    }
}
```
Given: std::thread::spawn inside workflow fn (NOT tokio::spawn)
When: lint_workflow_code() is called
Then: returns empty diagnostics (only L006 should catch this)

### test_no_false_positive_qualified_tokio_spawn
```rust
async fn helper() {
    // This is NOT tokio::spawn - it's some_other::spawn
    some_other::spawn(async { });
}
```
Given: path-qualified spawn that is NOT tokio::spawn
When: lint_workflow_code() is called
Then: returns empty diagnostics

### test_span_accuracy
```rust
impl WorkflowFn for MyWorkflow {
    async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> {
        let x = tokio::spawn(async {  // span should point here
            Ok(42)
        });
        Ok(())
    }
}
```
Given: tokio::spawn with known position
When: lint_workflow_code() is called
Then: Diagnostic span matches the exact byte offset of "tokio::spawn"

## Given-When-Then Scenarios

### Scenario 1: Detect tokio::spawn in simple workflow
Given: A Rust file containing an impl WorkflowFn with tokio::spawn in execute body
When: The linter runs on this source
Then:
- Returns exactly one Diagnostic with code L005
- The diagnostic severity is Error
- The message mentions tokio::spawn and workflow function

### Scenario 2: No false positives for non-workflow code
Given: A Rust file with tokio::spawn but no impl WorkflowFn
When: The linter runs on this source
Then:
- Returns empty Vec<Diagnostic>
- No error is returned

### Scenario 3: Multiple violations in same function
Given: A workflow function containing 3 tokio::spawn calls
When: The linter runs on this source
Then:
- Returns exactly 3 Diagnostics, each with code L005
- Each diagnostic has a unique span pointing to its respective tokio::spawn

### Scenario 4: Handles parse errors gracefully
Given: Invalid Rust source code that cannot be parsed
When: The linter runs on this source
Then:
- Returns Err(LintError::ParseError) with descriptive message
- Does not panic

### Scenario 5: tokio::spawn in nested contexts
Given: tokio::spawn inside match arms, if expressions, closures, loops within a workflow
When: The linter runs on this source
Then:
- Detects all tokio::spawn occurrences regardless of nesting depth
- Returns one Diagnostic per tokio::spawn found
