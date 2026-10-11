# Compound negative-control observation verifier v5

This private verifier retains v4's exact frozen path-alias handling and corrects the copied-input inventory check. The 74 frozen inputs contain the original source and Runner; replay-local copies live under `inputs/` and are separately pinned by `execution.copied_inputs`, not members of that 74-pin list.

For both source and Runner, the verifier checks the original fixture path and copy path, verifies each pin against disk, requires the original pin in the frozen input inventory, and compares bytes, SHA-256, and mode across original/copy pins. Replay commands and profile records then bind to the copied paths. No assertion requires those copied pins to appear in the original frozen-input inventory.

The earlier exact handling remains: 74 unique literal paths, 68 resolved paths, and only the six observed `/Library/...` versus `/Users/lordcasser/Library/...` JDK tool alias pairs. Each alias pair has identical bytes/SHA-256/mode; all pins are verified individually, and the post-run pin list must remain identical to the original list.

This candidate was syntax-checked with Python AST parsing and reviewed against the recorded execution JSON. The verifier itself has not been executed. No workspace, Git, Cargo, JDK, or product CLI was used.
