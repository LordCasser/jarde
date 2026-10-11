# Observation acceptor v3 preparation

Private fix for the v2 collector-argv order mismatch. The CLI option order now follows the frozen launcher’s explicit argument list rather than JSON object serialization order. Exact argument and config checks remain in place.

See `REVIEW.md` and `pins.json`. This acceptor is not executed during preparation; root should run it after full-workspace cleanup.
