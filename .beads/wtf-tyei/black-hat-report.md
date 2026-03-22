# Black Hat Review: POST /api/v1/workflows/validate

bead_id: wtf-tyei
bead_title: wtf-api: POST /api/v1/workflows/validate — workflow definition linting endpoint
phase: black-hat
updated_at: 2026-03-21T23:58:00Z

## Security Review

### Authentication/Authorization
- Not applicable - endpoint is intended for internal use
- No auth implemented - appropriate for MVP

### Input Validation
- ✅ JSON parsing via serde (fail on malformed JSON → 400)
- ✅ Rust source parsed via syn (fail on syntax error → 400)
- ⚠️ No size limit on source string (potential memory DoS)

### Error Handling
- ✅ Parse errors return 400 with message (no stack trace leakage)
- ✅ No panics in handler code
- ✅ Uses `?` and `Result` types properly

### Code Quality
- ✅ No unsafe code
- ✅ No unwrap/expect in hot path
- ✅ Proper error types (thiserror, anyhow)

### Potential Issues
- ⚠️ No request body size limit (could receive huge JSON)
- ⚠️ Pattern matching in comments could cause false positives (e.g., `// now()`)

## Conclusion

**STATUS: APPROVED** - Implementation is secure for intended use case.

Recommendations for future:
1. Add request body size limit (e.g., 1MB max)
2. Consider AST-based linting to avoid false positives in comments
