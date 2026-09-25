# Frozen patch method

"
        "Input is the permanent Java 8 class at `tests/fixtures/p3-floating-constants/v8/FloatingConstants.class`; "
        "its SHA-256 is checked before any experiment. `replay.py` also recompiles the checked-in source-only "
        "Java inputs and requires byte-for-byte equality with that class. Pool variants replace only the exact "
        "Float/Double entries including their tags. Operation variants replace exactly one complete Code "
        "attribute per method, with asserted original bytes and updated `max_stack` where needed. All generated "
        "classes live in a temporary directory and are removed when replay exits.

"
        "This is fixture/方案 evidence. It does not assert that current Jarde recovery restores these values.
