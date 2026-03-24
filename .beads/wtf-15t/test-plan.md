# Martin Fowler Test Plan: get_journal handler

## Happy Path Tests
- test_returns_journal_entries_sorted_by_sequence
- test_returns_correct_invocation_id_in_response
- test_returns_empty_entries_for_new_workflow

## Error Path Tests
- test_returns_400_when_id_is_empty
- test_returns_400_when_id_is_whitespace
- test_returns_400_when_id_missing_namespace
- test_returns_404_when_workflow_not_found
- test_returns_500_when_event_store_unavailable
- test_returns_500_when_replay_stream_errors

## Edge Case Tests
- test_handles_id_with_multiple_slashes
- test_handles_id_with_url_encoded_characters
- test_response_content_type_is_application_json

## Contract Verification Tests
- test_entries_sorted_ascending_by_seq
- test_all_entry_fields_present
- test_journal_response_validates

## Given-When-Then Scenarios

### Scenario 1: Retrieve journal for existing workflow
Given: A workflow with namespace "payments" and instance_id "01ARZ3NDEKTSV4RRFFQ69G5FAV" exists
When: GET /api/v1/workflows/payments/01ARZ3NDEKTSV4RRFFQ69G5FAV/journal is called
Then: Returns 200 with JournalResponse containing all journal entries sorted by seq

### Scenario 2: Request journal for non-existent workflow
Given: No workflow with the given ID exists in the event store
When: GET /api/v1/workflows/payments/NONEXISTENT/journal is called
Then: Returns 404 with ApiError containing "not_found"

### Scenario 3: Request journal with malformed ID
Given: An invalid ID format "invalid-id-without-namespace"
When: GET /api/v1/workflows/invalid-id/journal is called
Then: Returns 400 with ApiError containing "invalid_id"

## Test Implementation Status (2026-03-24)
- [x] Handler implemented in journal.rs
- [x] HTTP layer tests in journal_test.rs: given_empty_id_when_get_journal_then_bad_request, given_whitespace_id_when_get_journal_then_bad_request, given_id_without_namespace_when_get_journal_then_bad_request
- [x] Unit tests in journal.rs: parse_journal_request_id, sort_entries_by_seq
- [x] Content-type validation test
- [ ] Integration test with real event store data (requires NATS)
