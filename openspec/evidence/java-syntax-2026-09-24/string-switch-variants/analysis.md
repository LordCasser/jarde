# String switch: default between labels

This fixture extends the earlier String-switch variants with `default` in the middle of the source label sequence. The default arm has an effect and falls through to `case ""`; `case "prefix"` also falls through to `case "suffix"`. Other coverage includes the `"Aa"`/`"BB"` hash collision and shared destination, `"雪"`, one selector call, null, and a selector that throws.

## Frozen input

Compiled with `/usr/bin/javac 23.0.1 --release 8 -g:none`.

| File | SHA-256 |
| --- | --- |
| `StringSwitchMiddleDefault.java` | `2057ef55869f213c3f05c8b1761148fe6152941cd5d80f9835823831e7e60416` |
| `StringSwitchMiddleDefaultRunner.java` | `c7a2252052fce5423391955da4f37ed2cf0c89862cd39fdcf839a1e0e6e33ff5` |
| `StringSwitchMiddleDefault.class` | `fd6caaaac6c00799e218601c76a6dc133238a851d05672237219bfbebfb2a647` |
| `StringSwitchMiddleDefaultRunner.class` | `d624a2eb51d4d302f2bd8ae503373a3415dab0dc8a1cc4e3d2343c088aee9d1d` |

`javap.txt` records the lowering. `choose` calls `read` once at BCI 5, initializes the discriminator to `-1` at BCIs 9–10, computes `hashCode` once, then dispatches through a `lookupswitch` and collision-aware `equals` checks. The second `tableswitch` sends unmatched `-1` through its default edge to BCI 195. Its explicit but unassigned `case 2` also targets BCI 195. That block performs `add(2)` and falls through to the empty-string discriminator's BCI 199 block (`add(3)`). The `prefix` and `suffix` discriminators likewise enter adjacent blocks at BCIs 213 and 217.

## Three-way complete-class comparison

The original class, JADX 1.5.6 output, and Jarde current-worktree output were each compiled with `javac --release 8 -g:none` alongside an equivalent runner. All three executions used `java -Xverify:all`. The compile logs have exit code 0, and `original-diff.txt`, `jadx-diff.txt`, and `jarde-diff.txt` are empty.

The 18 output lines are identical. They show the shared collision result (`Aa` and `BB` both return 1), default-to-empty fallthrough (`other` returns 203), empty (`3`), Unicode (`4`), prefix-to-suffix fallthrough (`506`), suffix (`6`), and one `read` call for every successful selector. Null throws `NullPointerException` after one call. The throwing selector throws `IllegalStateException` after one call.

JADX 1.5.6 marks `choose` with `JADX WARN: Failed to restore switch over string` and keeps both the hash switch and discriminator switch. Jarde also presents the two integer switches (`hashCode`/`equals`, then `switch (local2)`) and compiles with the complete class source. Thus Jarde is behaviorally correct for this fixture; the remaining gap is source-shape recovery.

## Recovery boundary exposed

In JADX `SwitchOverStringVisitor.prepareMergedSwitchCases`, every explicit integer key in the second switch must be found in the collected string-to-discriminator map before the region can be replaced. Here key `2` is an unreachable gap between assigned case numbers: no string comparison assigns it, and the no-match initializer is `-1`, which takes the second switch's default edge. The gap key shares that default target. The visitor therefore rejects this otherwise well-formed string switch. Its all-or-nothing refusal is safe and preserves compilability, but the proof is stricter than the source semantics require.

A bounded improvement can discard only a proven unreachable gap key: the pre-switch discriminator is initialized to `-1`; the complete set of successful string comparisons assigns `{0, 1, 3, 4, 5, 6}` and no other path writes `2`; the explicit `case 2` and default have the same target; and the default target plus its fallthrough path is retained as the source default arm. Any additional write/use, distinct target, or unaccounted label must still refuse folding. This is evidence for a future 1.2 design decision, not a claim that 1.2's negative cases or implementation are complete.

## Tool identities

- JADX: `/opt/homebrew/bin/jadx`, version `1.5.6`
- Jarde CLI: `/private/tmp/jarde-string-switch-fixtures-target/debug/jarde-cli`, SHA-256 `cad07f5cb9a9f5235f8e737758c296ecc6bdb82fc61265af97497f14bd35c206`
- Exact decompiler text: `jadx/sources/defpackage/StringSwitchMiddleDefault.java` and `jarde.java.txt`

Root subsequently rebuilt the fixture source with `javac --release 8 -g:none` in a separate temporary directory and confirmed the class bytes equal the frozen `fd6caaaac6c00799e218601c76a6dc133238a851d05672237219bfbebfb2a647` input. It independently reran JADX on that class, compiled its default-package-normalized source and the saved Jarde source with the frozen runner, then executed all three with `java -Xverify:all`: 18/18 lines match, including `choose:other:203:calls=1`. The temporary directory was removed after the check.
