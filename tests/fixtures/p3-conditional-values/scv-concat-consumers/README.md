`ScvConcatConsumers.java` is compiled with `javac --release 8 -g:none -Xlint:-options`
(change `recover-scv-concat-consumers`; the patrol evidence is
`openspec/evidence/java-syntax-2026-10-01/em19-bitops-patrol/`).

Every `plain`/`boxedHead`/`doubleChain`/`singleTail` member stores a short-circuit
1/0 graph into a `boolean` local and consumes the local as a concatenation operand —
`StringBuilder.append:(Z)Ljava/lang/StringBuilder;` — in the four acceptance shapes:
the plain tail operand, the `""` head-position chain (which still lowers to `append(Z)`,
pinned by javap in the patrol evidence), two chains over one `append(Z)` each, and the
single-test second conditional living in the first chain's consumer block.

`ScvConcatReReadControls.java` (same compile command) holds `reRead`: the local's value
reaches a `boolean[]` store before the concatenation, so one read is not an explicit
Boolean consumer and the whole method keeps its degradation. The control is its own
class because a quoted member has no `return` statement to compile.
