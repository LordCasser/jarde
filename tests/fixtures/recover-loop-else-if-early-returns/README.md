# recover-loop-else-if-early-returns — frozen fixtures

The binary-search patrol's own shapes
(`openspec/evidence/java-syntax-2026-10-05/binary-search-twopointer-patrol/fixture/`) and this
change's own anchor and negatives, compiled by both javac legs.
`tests/recover_loop_else_if_early_returns.rs` reads exactly these bytes.

## Sources

`BS`, `CB` and `CB2` are the patrol's fixtures **verbatim** (`BS.java`/`CB.java`/`CB2.java`,
`cmp`-identical to the patrol's own copies):

- `BS` — the patrol's whole class: `bsearch` (the anchor this change recovers), `twoPtr` (the
  patrol's still-open critical anchor 16, whose quote this change does not touch), `fib`, `memo`
  and `main`;
- `CB` — the patrol's discriminating matrix: `loopElseIfRet` (the second anchor), and the three
  controls `loopElseIfNoRet` (ladder without an early return), `loopIfElseRet` (a single `if`/`else`
  with an early return) and `noLoopElseIfRet` (a ladder with no loop);
- `CB2` — `exitVal`, the same ladder whose exit consumes the loop variable (the patrol's third
  recovery claim: refused at HEAD, recovered by this change), and `exitVal2`, the same exit
  consumption without a ladder (unchanged control).

`LR2` and `LB` are this change's additions:

- `LR2` — the anchor written so that **every** member recovers (`bsearch` over an array plus a
  `sample()` builder and a plain `main`): its whole-class presentation is one compilable unit, which
  is what the replay strips and compiles as a whole class;
- `LB` — the boundary of the reading: `doubleLadder` (a two-level ladder) and `tryLadder` (an
  exception table crossing the ladder) keep the patrol's canonical-overlap refusal; `switchInArm` (a
  switch inside a ladder arm) and `firstArmRet` (the early return in the ladder's *first* arm) keep
  their cross-quote refusal; `forLadder` (a `for` header's ladder, where the update block is the
  join) and `switchLadder` (a switch that is the ladder's sibling) recover, and are pinned as such.

| source | SHA-256 |
| --- | --- |
| `BS.java` | `56c9d5a019210bd98e143db8a91087fa7a841235350ef90d2285af23bcab9926` |
| `CB.java` | `2ab2bc461cda0cca2ffa795a1b6d10c7a21aa85438aa37c401970ac014d7a078` |
| `CB2.java` | `df1a6be72e6a9fa6fc0576c7021913884ecdcb5bd320453d804cc49de1fbf802` |
| `LB.java` | `9a868310ff0f8b04bc08b93af9848734e090ed7179a642988387a6a4eea4e4e0` |
| `LR2.java` | `69f42e8f60b5bda1437012488aba85281da0c65aa6ae01ad7a73031d0200430d` |

## The two legs (one source, two compilers)

- `v8/` — **javac 23.0.1**, `javac --release 8 -Xlint:-options -d v8 *.java`;
- `v8-javac8/` — **real javac 8**, Corretto 1.8.0_432
  (`/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac`,
  `javac -version` → `javac 1.8.0_432`, **no `--release`**: the default target is Java 8),
  `javac -d v8-javac8 *.java`.

`v8/BS.class` is `cmp`-identical to `BS.class` inside the patrol's committed
`binary-search-twopointer-patrol/fixture/bs.jar` (1,737 bytes) — the anchor's bytes are the patrol's
own record. Both legs present byte-identical texts for every member this change pins.

| class file | bytes | SHA-256 |
| --- | ---: | --- |
| `v8/BS.class` | 1,737 | `79fc1c4adb7abbad37d318c945f3c3ef1e74fd0d2311348399a71d52aa6e0fec` |
| `v8/CB.class` | 1,243 | `9dacbb975b15857b886e88ae0796ebbb530bdc0294328065d298cdf61ff97d48` |
| `v8/CB2.class` | 950 | `78ad5e5542ed5350534f0f964b880aec9ecf4d2d0949788fe4102f25c60e3785` |
| `v8/LB.class` | 1,864 | `3413193ac6e5f53147756479dd4997addffdb56362fe7d41ec59ae9283955ac5` |
| `v8/LR2.class` | 932 | `28274edd197d53ad0e5554469640e2f0fb0e5a9d12efe6432f0ce9e794f00937` |
| `v8-javac8/BS.class` | 1,740 | `7fed29ce5b03e962873a69cc12d5d175288d87836001e02f654d2153f1efe0c6` |
| `v8-javac8/CB.class` | 1,243 | `09586136f335262905880456859166d4e9777d0c0b13206ffacff3e7f21e4149` |
| `v8-javac8/CB2.class` | 950 | `63de6eb5bfb10b50b0343cdfb71aae300ab722a3f8f29594d23b47d24c2ce89a` |
| `v8-javac8/LB.class` | 1,867 | `b525889d7e4bfaf6a72ae7609af24d09f931478fd6e50a12fc871643ea6c26b8` |
| `v8-javac8/LR2.class` | 932 | `6177ef501d79d5b95a49d768e2d2e2cb3f607f177b9ca0237d41d17c59294348` |

## The discriminating facts (javap, `v8` leg; both legs share the BCIs)

`CB.loopElseIfRet` — `25: iload 5; 27: iload_1; 28: if_icmpge 39` (the ladder's first test),
`39: iload 5; 41: iload_1; 42: if_icmple 53` (the second), `45–50: hi = m - 1; goto 56`,
`53: iload 4; ireturn` (the early-return leaf), `56: goto 7` (the latch block the two arms
regroup on). BCI 56 is the block the patrol's refusal named: the walk claimed it from the first arm
and re-entered it from the second.

`LB.doubleLadder` — the same layout one level deeper: its own latch (BCI 47) is claimed three
times, which is why the two-level shape keeps the refusal.

`LB.forLadder` — `for(int i = 0; …; i++)`: the arms jump to the update block, so the update block
is the ladder's join, and the recovered text writes the increment as the loop body's last
statement.

## The tests that read them

`tests/recover_loop_else_if_early_returns.rs`:

- not ignored: both legs render every fixture once and pin the members' whole texts — the two
  anchors and `CB2.exitVal`, the three controls and `CB2.exitVal2` byte for byte, the four refused
  negatives and the two further recovered cells, plus the refusal counts in `LB`;
- ignored (`cargo test -- --ignored`, needs a JDK on `PATH`): `LR2`'s **whole-class** presentation
  is stripped (comment lines dropped), compiled by the installed `javac --release 8` **and** by a
  real javac 8 when one is present (`JARDE_JAVAC8`, else the Corretto path above), run with
  `-Xverify:all` and compared with the fixture's own class run (`2/-3`); each patrol anchor's
  recovered **method** text is compiled beside the fixture's own call sequence (their `main` is
  refused, so their whole-class strip cannot compile by design) and compared with the values the
  fixture class itself prints — `BS` `2/-3`, `CB` `1`, `CB2` `-3`, `LB` `0/-1`.
