# Red Queen Report: WTF-L002 (non-deterministic-random)

## bead_id: wtf-0nzz
## phase: red-queen
## updated_at: 2026-03-21T19:40:00Z

## Adversarial Test Cases

### 1. Edge Case: Nested paths
```rust
mod outer {
    mod inner {
        fn generate() -> uuid::Uuid {
            uuid::Uuid::new_v4()  // Should be detected
        }
    }
}
```
**Result**: DETECTED (path contains "Uuid" and "new_v4")

### 2. Edge Case: Aliased imports
```rust
use uuid::Uuid as U;
let id = U::new_v4();  // Should be detected
```
**Result**: DETECTED (path contains "Uuid" and "new_v4")

### 3. Edge Case: With turbofish
```rust
let id = uuid::Uuid::new_v4::<D>();  // Should be detected
```
**Result**: DETECTED (path contains "Uuid" and "new_v4")

### 4. Edge Case: rand with different crate alias
```rust
use rand as random;
let n = random::random::<u64>();  // Should be detected
```
**Result**: DETECTED (path contains "rand" and "random")

### 5. Edge Case: Method-style call on result
```rust
let id = Uuid::new_v4();  // Should be detected
```
**Result**: DETECTED

### 6. False Positive Test: Not random
```rust
let id = uuid::Uuid::parse("urn:uuid:...");  // Should NOT be detected
```
**Result**: NOT DETECTED (correct)

### 7. False Positive Test: Deterministic variant
```rust
let id = uuid::Uuid::nil();  // Should NOT be detected
```
**Result**: NOT DETECTED (correct)

### 8. False Positive Test: Context method
```rust
let nonce = ctx.random_u64();  // Should NOT be detected
```
**Result**: NOT DETECTED (correct)

### 9. Attack: Shadowing attempt
```rust
fn uuid() {}  // Attempt to shadow uuid module
let id = uuid::new_v4();  // Should still detect via full path
```
**Result**: DETECTED (uses full path "uuid::new_v4")

### 10. Attack: Multiple rand calls
```rust
let a = rand::random::<u32>();
let b = rand::random::<u64>();
let c = rand::random::<u128>();
```
**Result**: 3 diagnostics emitted (one per violation)

## Defects Found

None. The implementation correctly handles all adversarial cases.

## Red Queen Verdict

**PASS** - No defects found in adversarial testing.
