# do-while body transfer fixtures

The `DoWhileCore` subject and runner are copied from the frozen Java 8 audit at
`openspec/evidence/java-syntax-2026-09-22/do-while/core/`. Compile all three Java sources with
`javac --release 8 -g:none`; the subject class must match SHA-256
`9339878366306e65530aee489e15ca5ead92c4ff637b614141d9a8e552cb7a60`.

`withContinue(I)I` has a conditional edge from BCI 10 to the latch at 24. The path skips the
`trace` update at BCI 13–21 but still tests the loop at BCI 24. `withBreak(I)I` has a conditional
edge from BCI 10 to the loop exit at 29, skipping the body remainder and latch. The runner fixes
all seven original outputs, including `continue:4=134:trace=134` and `break:5=12:trace=12`.

`DoWhileSwitchBoundary.switchBreak(I)I` is the negative control for an unmarked `break` nested
inside a `switch`: the switch edge from BCI 28 goes to BCI 38, still inside the loop; the loop
latch at BCI 48 goes back to BCI 4 and the actual loop exit is BCI 51. Its pinned class SHA-256 is
`80b32c4b68f6409f236f4f7e049966dd30576602a82ac71224c56217b51b4fc8`. Expected runner outputs are
`switch:2=199:trace=0` and `switch:3=19939:trace=0`.

The ignored integration test is the RED gate for the upcoming transfer implementation. It first
checks the fixture hashes, Java 8 rebuild, original execution, `javap` edges and BCI source maps,
then asserts the not-yet-implemented structured output. The ordinary nested-switch test stays
active and asserts conservative fallback with no emitted unmarked `break;`.
