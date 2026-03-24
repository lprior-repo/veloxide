# Martin Fowler Test Plan: handle_procedural_msg Error Handling

## Happy Path Tests
- `test_returns_ok_when_procedural_handler_succeeds` - Given valid dispatch, When handle_procedural_msg is called, Then returns Ok(())
- `test_returns_ok_when_sleep_handler_succeeds` - Given valid sleep request, When handle_procedural_msg is called, Then returns Ok(())

## Error Path Tests
- `test_returns_error_when_dispatch_handler_fails` - Given dispatch returns error, When handle_procedural_msg is called, Then error is propagated
- `test_sends_error_via_reply_channel_when_dispatch_fails` - Given dispatch fails with reply channel, When handle_procedural_msg is called, Then error is sent via reply
- `test_logs_error_when_handler_fails` - Given handler returns error, When handle_procedural_msg is called, Then tracing::error! is called
- `test_returns_error_when_sleep_handler_fails` - Given sleep returns error, When handle_procedural_msg is called, Then error is propagated
- `test_returns_error_when_now_handler_fails` - Given now returns error, When handle_procedural_msg is called, Then error is propagated
- `test_returns_error_when_random_handler_fails` - Given random returns error, When handle_procedural_msg is called, Then error is propagated
- `test_returns_error_when_wait_for_signal_fails` - Given wait_for_signal returns error, When handle_procedural_msg is called, Then error is propagated
- `test_returns_error_when_completed_handler_fails` - Given completed handler returns error, When handle_procedural_msg is called, Then error is propagated
- `test_returns_error_when_failed_handler_fails` - Given failed handler returns error, When handle_procedural_msg is called, Then error is propagated

## Edge Case Tests
- `test_handles_missing_reply_channel_on_error` - Given handler fails with no reply channel, When handle_procedural_msg is called, Then error is propagated via return value only
- `test_handles_unexpected_message_type` - Given unknown message variant, When handle_procedural_msg is called, Then returns ActorProcessingErr

## Given-When-Then Scenarios

### Scenario 1: Error propagation with reply channel
**Given:** A `ProceduralDispatch` message with a reply channel  
**When:** `handle_dispatch` returns an error  
**Then:**
- The error is logged via `tracing::error!`
- The error is sent via the reply channel
- `handle_procedural_msg` returns `Ok(())` (error handled via channel)

### Scenario 2: Error propagation without reply channel
**Given:** A `ProceduralDispatch` message without a reply channel  
**When:** `handle_dispatch` returns an error  
**Then:**
- The error is logged via `tracing::error!`
- The error is propagated via return value
- `handle_procedural_msg` returns the error

### Scenario 3: Successful dispatch
**Given:** A `ProceduralDispatch` message with valid inputs  
**When:** `handle_dispatch` returns `Ok`  
**Then:**
- `handle_procedural_msg` returns `Ok(())`
- No error logging occurs

## Contract Verification Tests
- `test_precondition_state_valid` - Verifies state is not null/valid when handler executes
- `test_postcondition_error_never_silent` - Verifies at least one error reporting mechanism is always used
- `test_invariant_error_logging` - Verifies tracing::error! is called on any handler error
