# Full-class integer constant candidate collector v2

`prepare-candidate-luna-v2.py` retains the v1 collection plan and adds direct
checks for actual BLAKE3 input identity, complete physical field/method
identity, and exact source-map BCI coverage against the saved `javap` inventory.
It also checks the intended positive and negative integer-array name cases at
their physical `Field` and `MethodPoint` anchors. The five controls require
`blake3==1.0.11` at runtime; root can provide it with `uv run --no-project
--with blake3==1.0.11 python -B ...`.

The metadata pin sets are exact: ten product sources, four test sources, and
the sixteen canonical paths listed in the script. The required four CLI and
metadata arguments remain runtime supplied, so this preparation does not
assume a not-yet-frozen candidate build. Output is
`candidate-full-class-root-v2/`; existing evidence is never overwritten.

This script is prepared for root review only. It has not been run and has not
invoked Java, JADX, Cargo, Git, or the candidate CLI. The three array-fill
oracles remain reused from the separately accepted baseline; they are not
claimed as freshly compiled in this collection.
