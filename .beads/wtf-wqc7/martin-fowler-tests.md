# Martin Fowler Tests: WTF-L006 std::thread::spawn

bead_id: wtf-wqc7
bead_title: wtf-linter: WTF-L006 std::thread::spawn in workflow function
phase: test-plan
updated_at: 2026-03-21T00:00:00Z

## Test Cases

### Given-When-Then Format

#### Test 1: thread::spawn detection
**Given** a workflow function body containing `std::thread::spawn(|| { ... })`
**When** `check_l006_thread` is called
**Then** a LintDiagnostic with code "WTF-L006" is appended to diagnostics

#### Test 2: thread::sleep detection
**Given** a workflow function body containing `std::thread::sleep(std::time::Duration::from_secs(1))`
**When** `check_l006_thread` is called
**Then** a LintDiagnostic with code "WTF-L006b" is appended to diagnostics

#### Test 3: nested spawn calls
**Given** a workflow function body with nested `thread::spawn` calls
**When** `check_l006_thread` is called
**Then** each spawn call generates a separate diagnostic

#### Test 4: no false positive on ctx.sleep
**Given** a workflow function body using `ctx.sleep()` (not std::thread::sleep)
**When** `check_l006_thread` is called
**Then** no WTF-L006 or WTF-L006b diagnostics are generated

#### Test 5: full integration - all 6 rules
**Given** a source file with all 6 violation types (including thread::spawn and thread::sleep)
**When** `lint_workflow_source` is called
**Then** at least 6 diagnostics are returned with correct codes including WTF-L006 and WTF-L006b

#### Test 6: no panic on empty body
**Given** an empty workflow function body
**When** `check_l006_thread` is called
**Then** the function returns normally with zero diagnostics

## Edge Cases
- Empty function body (should return 0 diagnostics)
- Multiple spawn calls in same function (each generates diagnostic)
- Spawn inside a loop or conditional
- Fully qualified std::thread::spawn vs use std::thread
