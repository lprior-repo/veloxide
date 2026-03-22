# Red Queen Report: WTF-L005 tokio-spawn-in-workflow

## Adversarial Test Cases

### Test Case 1: Nested tokio::spawn in conditional
```rust
impl WorkflowFn for MyWorkflow {
    async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> {
        let _ = if condition {
            tokio::spawn(async { do_work().await; })
        } else { 42 };
        Ok(())
    }
}
```
**Status**: ✅ Detected (test_violation_nested_tokio_spawn passes)

### Test Case 2: Multiple tokio::spawn in same function
```rust
impl WorkflowFn for MyWorkflow {
    async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> {
        tokio::spawn(async { println!("first"); });
        tokio::spawn(async { println!("second"); });
        Ok(())
    }
}
```
**Status**: ✅ Both detected (test_multiple_tokio_spawns passes)

### Test Case 3: tokio::spawn inside closure within workflow
```rust
impl WorkflowFn for MyWorkflow {
    async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> {
        let numbers = vec![1, 2, 3];
        numbers.iter().for_each(|_| {
            tokio::spawn(async { println!("in closure"); });
        });
        Ok(())
    }
}
```
**Status**: ✅ Detected (test_tokio_spawn_in_closure passes)

### Test Case 4: Non-tokio spawn (std::thread::spawn)
```rust
impl WorkflowFn for MyWorkflow {
    async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> {
        let handle = std::thread::spawn(|| { println!("thread"); });
        Ok(())
    }
}
```
**Status**: ✅ Not flagged (only L006 should catch this)

### Test Case 5: Qualified path that's not tokio
```rust
impl WorkflowFn for MyWorkflow {
    async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> {
        some_other::spawn(async { });
        Ok(())
    }
}
```
**Status**: ✅ Not flagged (test_no_false_positive_qualified_tokio_spawn passes)

### Test Case 6: tokio::spawn outside workflow function
```rust
async fn helper_function() {
    tokio::spawn(async { println!("helper task"); });
}
```
**Status**: ✅ Not flagged (test_no_false_positive_outside_workflow passes)

## Conclusion
All adversarial test cases pass. The implementation correctly detects tokio::spawn in workflow functions while avoiding false positives on similar patterns outside workflow contexts.
