# EM-01 declaration-only member pair

`Shape.java` is byte-for-byte the fixed EM-01 input. `javac --release 8 -g:none` yields the three frozen Java 8 class files:

| Class | SHA-256 |
| --- | --- |
| `Shape.class` | `8a8bf4ce6d81b49e71106711afcab52f29e786c798d40b27c359b843f0f54185` |
| `Shape$A.class` | `7d58c82f0ff0a9e143774ec0bd4514660363e08640a175848640199a428646fb` |
| `Shape$I.class` | `b311d5eb6eeead20d16e28e77a007ab035e587b2d3e5f85d3370efaf549e574b` |

The root's `InnerClasses` rows occur in `A`, `I` order. Their flags are `0x0409` and `0x0609`; both child self rows agree. `A` has one default constructor and one no-Code abstract method. `I` has exactly two public abstract no-Code methods and no constructor. The pre-change physical reports are in `openspec/evidence/java-syntax-2026-09-27/em01-declarations/baseline/`: both children are queryable, while root `member_family` refuses at `direct member is not a source-spellable named class` and emits neither declaration.

Run `replay.sh JARDE_CLI JADX_BIN JADX_CHECKOUT OUTPUT_DIR`. It checks pinned JADX HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`, recompiles original, JADX and Jarde full `Shape` sources with the same external `Runner.java`, and compares `java -Xverify:all` output `2:1`. The Jarde source must contain one `A`, one `I`, the three abstract method semicolon declarations and a source-spelled `A()` constructor. Both physical child reports remain independently queryable.

`PairNeighbor.java` deterministically reproduces seven verifier-valid mutations: a conflicting `I` self row; a default or static `I` method with Code; an extra `I` field or Signature; a third direct root row; and a root constructor use of `I.class`. A missing-`I` archive is the eighth neighbor. The script checks that none emits either nested declaration. Rust tests also check pair state, exact physical order, six derived spans and anchors, child report identities, and output-budget refusal. Generic `Generic.A` remains outside this certificate.
