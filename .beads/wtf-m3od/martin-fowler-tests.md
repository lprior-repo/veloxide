# Martin Fowler Test Plan: Wire definition storage and registry loading

## Happy Path Tests
- test_loads_all_definitions_from_kv_on_startup
- test_returns_empty_vec_when_kv_bucket_is_empty
- test_deserializes_valid_workflow_definitions
- test_populates_registry_with_loaded_definitions

## Error Path Tests
- test_returns_error_when_kv_scan_fails
- test_skips_malformed_json_definitions_with_warning
- test_skips_entries_that_fail_to_fetch
- test_handles_kv_connection_failure

## Edge Case Tests
- test_handles_large_number_of_definitions
- test_handles_definitions_with_special_characters_in_names
- test_deduplicates_if_same_key_appears_multiple_times

## Contract Verification Tests
- test_all_loaded_definitions_are_valid
- test_registry_keys_match_kv_keys
- test_loading_does_not_modify_kv_state

## Given-When-Then Scenarios

### Scenario 1: Load definitions on startup
Given: KV bucket contains 3 valid workflow definitions
When: load_definitions_from_kv is called during serve startup
Then: Returns Vec with 3 (key, WorkflowDefinition) tuples

### Scenario 2: Empty KV bucket
Given: KV bucket exists but contains no definitions
When: load_definitions_from_kv is called
Then: Returns empty Vec and logs info message "No workflow definitions found in KV"

### Scenario 3: Mixed valid and invalid entries
Given: KV contains 2 valid definitions and 1 malformed JSON entry
When: load_definitions_from_kv is called
Then: Returns Vec with 2 valid definitions, logs warn for malformed entry

### Scenario 4: All entries malformed
Given: KV bucket exists but all entries fail to deserialize
When: load_definitions_from_kv is called
Then: Returns empty Vec, logs warn for each failed entry

## Test Implementation Status (2026-03-24)
- [x] Registry tests added: new_registry_is_empty, register_and_get_definition, get_nonexistent_definition_returns_none, register_multiple_definitions, definition_keys_are_case_sensitive, replacing_definition_overwrites
- [ ] load_definitions_from_kv tests require NATS KV mock - pending
