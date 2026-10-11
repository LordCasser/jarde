# Private clean-main auditor v3 preparation

This is a read-only preparation copy derived from frozen v2. The audit logic change updates the final-build producer path and digest to root's new adapter; the output schema tag advances to v3 so future receipts identify this prepared auditor. `audit-clean-main.patch` records the diff.

The actual workspace run and independent local acceptance are immutable inputs in `pins.json`: execution SHA `91998248…e4c733` and acceptance SHA `712cfdde…ea4b2d`. The regression receipt is root-v4 and remains schema v3; its candidate CLI path is extracted from that receipt. The final-build producer is `/private/tmp/jarde-hoisted-iterator-final-build-root-v2.py` SHA `ce22ee46…100ecc3`, with acceptor `/private/tmp/jarde-hoisted-iterator-accept-final-root-v2.py` SHA `a71e63ac…701a92`. The runtime CLI remains the actual v2 CLI SHA `8bdb868a…dfc9a`; the final build and regression CLI paths are receipt-derived and their bytes are compared.

Root is still running final-build-v4. Its execution/acceptance paths and SHA values are deliberately unresolved. The auditor requires all five receipt/acceptance path and SHA pairs as explicit command-line arguments; do not add unknown values to source constants.

The previous final-build-v3 attempt failed because `command_stream` requires stream paths relative to the repository while its output was under `/private/tmp`; retain that failed receipt as history. This v3 auditor has not been executed. Static checks parsed the Python AST and confirmed the source delta contains only the producer path, producer SHA and schema label. No Cargo, Git, worktree, resource, own-CI or clean-main audit command was run.
