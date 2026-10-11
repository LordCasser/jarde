# Private fixed and controls capture acceptors

`accept_current.py` independently checks the fixed `TestNestedLoops` full-class capture. `accept_controls.py` independently checks the positive/refusal controls capture. Both are read-only with respect to source, toolchain, and shared OpenSpec files; on success they write acceptance summaries only into this private directory and refuse to overwrite them.

Both acceptors bind the frozen focused receipt (`hoisted-iterator-focused-root-v2`, status `passed_focused_candidate_only`) and the wrapper-generated derived candidate-build subreceipt. They verify that the derived subreceipt records the parent receipt's exact path/SHA/schema/status, candidate CLI path/size/SHA/mode, and the successful Cargo build at command index 6 with its raw stdout/stderr pins. The original parent receipt remains unmodified.

The current capture is pinned at 29 commands/58 raw streams; it requires the original class set of `cf11.UpstreamNestedLoops` plus `cf11.Runner`, two original runtimes, and eight generated runtime legs with exact class set and stdout/stderr matching the corresponding JDK oracle. It independently checks four Jarde source/report profiles, exact Javap/physical BCI maps (45 physical instructions in `test` per profile), complete map coverage, and default/all map parity. The rendered Jarde presentation currently has one `while`, one enhanced `for`, and the `ordinary_generic_source_unproved` marker per profile; behavioral acceptance remains explicitly `not_claimed`.

The controls capture is pinned at 43 commands/86 raw streams. It preserves the observed four JADX runtime divergences and written-gap exception contrast, four positive isolated Jarde compile/runtime matches, and eight Jarde compile failures with no runtime. It keeps the four `sumWithWrittenGap` source-map gaps explicit. Positive generated output is checked for one outer `while` and one inner `for`; refusal output remains without either loop. Neither acceptor promotes the captured quality gaps to behavioral acceptance.

Both reconstruct the live 4,729-file Rust source closure, validate source/tool pins, frozen Java inputs, exact command arguments, each command's exit/guard/raw streams, 5 GiB free / 1 GiB target / one-second guard evidence, and target absence. This is capture acceptance for these fixed fixtures only; it does not claim whole-workspace validation.

All files and upstream inputs are bound in `pins.json`. Both acceptor scripts passed static `ast.parse` and `symtable.symtable` checks. The acceptors themselves were not executed in this preparation.
