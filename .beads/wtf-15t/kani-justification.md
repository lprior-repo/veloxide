# Kani Justification

Kani is unneeded for `get_journal` as the state handling is linear sequential event parsing. No loops altering in-flight invariants are present that demand Kani verification models.