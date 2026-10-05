# recover-platform-interface-argument-widening — frozen fixtures

The platform-interface-widening patrol's own shapes
(`openspec/evidence/java-syntax-2026-10-05/platform-interface-widening-patrol/`) and the
comparator-anon patrol's critical anchor and own-interface control
(`openspec/evidence/java-syntax-2026-10-05/comparator-anon-patrol/`), compiled by both javac legs.
`tests/recover_platform_interface_argument_widening.rs` reads exactly these bytes.

## Sources

`CP` (the anonymous-`Comparator` argument, `CP$1`/`CP$User` its companions), `AH`/`AC` (the
same call with a top-level named implementer: the patrol's argument against the `$` companion
spelling being the cause) and `AN` (the own-interface family whose four positions — return,
argument, static-field initializer, local — are the zero-regression control) are the patrols'
fixtures verbatim. `IS` and `AW` are this change's additions:

- `IS` — the patrol's isolated age-ordering shape (the one whose strip printed `[30, 10, 20]`
  before this change and `[10, 20, 30]` after it), written out as a compilable class;
- `AW` — the two-sided boundary of the walk: `viaNamedPlatform` proves the platform interface a
  snapshot class header **names** (`AW$Work implements java/lang/Runnable`), and `viaAbsent`
  keeps the refusal where the target appears in no snapshot header at all (`AW$MyErr extends
  java.lang.Exception`, and nothing in this snapshot states the platform chain up to
  `java.lang.Throwable`).

| source | SHA-256 |
| --- | --- |
| `AC.java` | `a9cecb4dc913533d28eea6ff3a5a69600b943f68cfa08ce0e76a1bd4e84d25e3` |
| `AH.java` | `87c8636e9edd65e1452cf16a554a8288ff273a4864fe364917aed18e18f5d07f` |
| `AN.java` | `881625ede468914c1869efe7299bf35ccad4c1bcc9975dd5090efcb82d936d76` |
| `AW.java` | `5b5b33252557f5c5581be83f5696f5ce973565cc403df550fb818bf158857a90` |
| `CP.java` | `37eef294ce5e849783e1d3ee35b264d6ef62369c4b3dadc0c653fdddf73031f8` |
| `IS.java` | `125de5aacb6abb6dfe17bfcdff3484d2400651758b05caa1a86785b39db6b26c` |

## The two legs (one source, two compilers)

- `v8/` — **javac 23.0.1**, `javac --release 8 -Xlint:-options -d v8 *.java`;
- `v8-javac8/` — **real javac 8**, Corretto 1.8.0_432
  (`/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac`,
  `javac -version` → `javac 1.8.0_432`, **no `--release`**: the default target is Java 8),
  `javac -d v8-javac8 *.java`.

Both legs present byte-identical texts for every member this change pins
(`CP.byAnon`, `AH.byTop`, `IS.byAnon`, `AN.returned`/`asArg`/`<clinit>`/`localVar`,
`AW.viaNamedPlatform`/`viaAbsent`).

| class file | bytes | SHA-256 |
| --- | ---: | --- |
| `v8-javac8/AC.class` | 539 | `7faaba24b8e337bbfbcaa97b0a3179383208f4d1e68553b82ded09a01d8052e3` |
| `v8-javac8/AH.class` | 929 | `2762c84781440ea676bc71d8a04ec040a382bdb76d1c0a8c50b7235a8801fb9e` |
| `v8-javac8/AN$1.class` | 358 | `3d428ec69f27205faccac17f4e987e541fc30791a998c97843bfaa617a07a713` |
| `v8-javac8/AN$2.class` | 343 | `6340645f53b4c58cd5a838f10dd6410386e686470454dac0a79da2fd1c4646b3` |
| `v8-javac8/AN$3.class` | 330 | `e920a0ca58cc4674d25c97ed10f631d519fef124f3ead1dd4aa1621ce03b648f` |
| `v8-javac8/AN$4.class` | 352 | `4043bf25b423a14e47593f271d00c6bf2506f6593bd296047fec0f17b7c37c2f` |
| `v8-javac8/AN$Op.class` | 155 | `a28699e8be72f950d7e427cf415eb081d4fd850fca560a19669c40db393f5ac3` |
| `v8-javac8/AN.class` | 1,219 | `296cad25d424657027ea77adb0211c797bce1b9d4bb5ae6ead686c1d634d5a05` |
| `v8-javac8/AW$MyErr.class` | 234 | `40af78a3d1bf3ee8bcb37d94f3026c9b0f80335059912988f681b96d066bcfea` |
| `v8-javac8/AW$Work.class` | 300 | `9493ca6bd68d3873fbd64ee1f0770ad34d5fa141d318d40e59814d2ec66e20b3` |
| `v8-javac8/AW.class` | 709 | `26924d1d4a107a679a6621bd7d5db5e477afe94ca83128c480568cb00be8526d` |
| `v8-javac8/CP$1.class` | 633 | `a1511243d82dd5df674bea7014c1f2861a4da8df2c96412de2cdd0021892947d` |
| `v8-javac8/CP$User.class` | 710 | `5424d7bafc777d1a6585d06821d464f58012a863f638d3f5ead568959c4ddb28` |
| `v8-javac8/CP.class` | 2,804 | `f164df2e70613a38221fd6d3e8bbc775ac3e19ddca89bb6aeb99d17889bd5d24` |
| `v8-javac8/IS$1.class` | 480 | `b67d0277872ef258be5666f24d9e0bbc509603c52b56607447594fa116879f77` |
| `v8-javac8/IS$User.class` | 522 | `317b8d7ec500ba12911224cfda6cd7b4bfd5e151724927ce1bda3563ed052e69` |
| `v8-javac8/IS.class` | 881 | `7b5f00a601a040a6490e921ae430697b0764bb2303b38774d291a899a12d424e` |
| `v8/AC.class` | 573 | `dafb7d91300d2462fbdce03ae495ea36a3e63b38845853d07974a81e3be4f9b4` |
| `v8/AH.class` | 929 | `a1adf42a2a14fcf133a9f8bba92204ac0165edc8b7fea64102c4a911b9ed0144` |
| `v8/AN$1.class` | 358 | `3c11f88bab1d5c2f12d4cd4e5776df5da6ec768ccdef22358576c67ca55d6a8e` |
| `v8/AN$2.class` | 343 | `97c9324f956526ed3566d11486c3c6468d7463ddef4e59eb10a2b5fef186988e` |
| `v8/AN$3.class` | 330 | `b2498fb5d5ed8e582f8cc8aa93f1bd0c176f5a08d0c6727b846d228ba8ed43a1` |
| `v8/AN$4.class` | 352 | `5b0c4d71a28989e31fb817d80b739ac0130060023335b4f093229c12114e5f92` |
| `v8/AN$Op.class` | 155 | `fa094a2010a5e0a4391ec995366bfc7d2ccdf866d6bf61f3eae1cc809f5d4ce3` |
| `v8/AN.class` | 1,219 | `d62742bf8e657733196835a1dbeff381f6925ec292ab9272a1f6a269e2f39246` |
| `v8/AW$MyErr.class` | 234 | `49970d079505ebc5233cd9ee4edf7c1d9e601fa839390baaebe566d6f5959488` |
| `v8/AW$Work.class` | 300 | `64e6c9bc53403542135f08d21c16295a172a8d8d9b8443e48f2677d16a95c299` |
| `v8/AW.class` | 709 | `8eeefde8c67ba7f6ca9879fc01be222a71c29b290d27cebd709d7dfc7aeaa168` |
| `v8/CP$1.class` | 667 | `2031d7c257e8f37bf643ff0afaeac68a851cdb20ac0ef4052fe9155e6429f938` |
| `v8/CP$User.class` | 710 | `57754c8fe3797bbee4f39c3a8e6e18c73ae94b3cd95abea5f98b80c9e5ad696e` |
| `v8/CP.class` | 2,801 | `c4aa7374390457c532b2353fb4ff94a1b2417c8851a8a6424f0262f6178a3651` |
| `v8/IS$1.class` | 480 | `c6fe1b72db8e152925462c8919d0088786e62359cb8b2626777462e111d78e89` |
| `v8/IS$User.class` | 522 | `e8e111af5a01841a272f752566134187c43474bee26e92472e782b4eca47db04` |
| `v8/IS.class` | 881 | `2074b02ebd8c6dad81abc3bc82d90257b2e133b90eee0aea26bdee6b61e81903` |

## The discriminating facts (javap, `v8` leg; both legs share the BCIs)

`CP.byAnon` — `9: aload_1; 10: new CP$1; 13: dup; 14: invokespecial CP$1.<init>();
17: invokestatic Collections.sort:(Ljava/util/List;Ljava/util/Comparator;)V`: BCI 17 is the
call the walk's target step decided. `CP$1`'s own header is `class CP$1 implements
java.util.Comparator` — the name the walk reaches, and the JDK interface whose header this
snapshot does not hold.

`AH.byTop` — the same call over `AC`, a **separately declared top-level** class of the same
container (`class AC implements java.util.Comparator`): the argument's type carries no `$`,
which is what the patrol used to rule the companion spelling out.

`IS.byAnon` — the raw form (`class IS$1 implements java.util.Comparator`, one `compare`), whose
recovered call is what the replay strips, compiles and runs for `[10, 20, 30]`.

`AW.viaNamedPlatform` — `0: new AW$Work; 3: dup; 4: invokespecial AW$Work.<init>();
7: invokestatic runRunnable:(Ljava/lang/Runnable;)V`; `AW$Work`'s header states
`implements java/lang/Runnable`. `AW.viaAbsent` — the same shape over `AW$MyErr`, whose header
states `extends java/lang/Exception` and nothing else: the required `java.lang.Throwable` is not
named by any snapshot header, so BCI 7 keeps the refusal verbatim.

## The tests that read them

`tests/recover_platform_interface_argument_widening.rs`:

- not ignored: both legs render every fixture once and pin the members' whole texts (the three
  presented calls, the four own-interface positions, the proved platform name and the one
  refusal), plus the count of refusals in `AW`;
- ignored (`cargo test -- --ignored`, needs a JDK on `PATH`): `CP`, `AH`/`AC`, `IS` and `AN` are
  stripped (comment lines dropped, every companion written as its own source unit under the
  binary name its class file states), compiled by the installed `javac --release 8` **and** by a
  real javac 8 when one is present (`JARDE_JAVAC8`, else the Corretto path above), run with
  `-Xverify:all`, and compared with the fixture's own class files' run —
  `[bo:30, al:40, al:20]/[al:20, bo:30, al:40]/[a, bb, ccc]`, `[a, bb, ccc]`, `[10, 20, 30]` and
  `11/42/9/25`. `AW` is never compiled: its `viaAbsent` refusal is the point.
