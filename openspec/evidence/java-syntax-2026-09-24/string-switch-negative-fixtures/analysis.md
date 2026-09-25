# String-switch candidate refusal cases

These are verifier-valid Java 8 classes whose methods resemble javac's two-stage String-switch lowering while violating one candidate-ownership invariant each. They test whether a future fold refuses unsafe patterns and leaves ordinary integer control flow intact. Both source files were compiled with `javac --release 8 -g:none`; no class-file patching was used.

## Frozen inputs

| Source | SHA-256 | Class (major 52) | SHA-256 |
| --- | --- | --- | --- |
| `ExtraHashUse.java` | `d1d5ea0f12ce735ec191508db1bac85755e95335b4e576ac3cf222af4a9e65de` | `ExtraHashUse.class` | `076e908fcc980f54a6aa09d49a1b63ebaeb6f9306431a0f8b9fad829daa910c5` |
| `WrongHashBucket.java` | `c4abe144240c6cf69981c6327541f3fa8a900ec00457c8fa68656dea05304f54` | `WrongHashBucket.class` | `f0a22997ff6974a252ab5279b2aa1f07cb315ea2cde0b5e4dccdf443cd22dc16` |

Runner inputs are also frozen: `ExtraHashUseRunner.java` SHA-256 `fb1e514c75a7fb4b7c85caac2e30fb95a8fff90cfef09a8356538e2170f7ffec`; `WrongHashBucketRunner.java` SHA-256 `cce8cc510e30359cecd7226bb8de97ff0b6b0ef3617cb10e75475de87f653714`.

Both full `javap -v -c -p` outputs are checked in. `java -Xverify:all` accepts original and recompiled Jarde classes. The fixtures each call `read` once and cover normal and null inputs. The independent-use runner includes `"a"` (hash 97), which makes the parity bit observable as 1.

## Independent hash consumer

In `ExtraHashUse.choose`, BCI 6 computes `String.hashCode`; BCI 17 uses it for the candidate hash `lookupswitch`, while BCIs 10–13 independently use the same integer as `parity = hash & 1`. The final discriminator switch is at BCI 88, and the method returns its result plus `parity`. Folding the hash result away with the comparison subgraph would lose a real data consumer.

Jarde retains both integer switches and the parity calculation. Its complete class source recompiles under `--release 8`; its six `-Xverify:all` output rows match the original exactly (`Aa=10`, `BB=20`, `abc=30`, `x=40`, `a=41`, null throws `NullPointerException`, with one `read` call each).

JADX 1.5.6 prints a cleanup warning, rewrites the method as `switch (String)`, but emits `int iHashCode = r0.hashCode() & 1;` with no declaration for `r0`. The resulting complete class fails Java 8 compilation (exit 1) at line 27: `cannot find symbol: variable r0`. This is a concrete source-compilation failure from removing the hash/discriminator SSA subgraph despite its independent consumer. No JADX runtime comparison is claimed for this uncompilable output.

## Wrong hash bucket

In `WrongHashBucket.choose`, the first `lookupswitch` at BCI 13 routes `2112` to BCI 54, where it compares against `"abc"` (hash `96354`), and routes `96354` to BCI 40, where it compares against `"Aa"` (hash `2112`). The class is valid, but these comparisons are intentionally placed in the wrong buckets. `"Aa"` and `"abc"` therefore both reach the discriminator default and return 40.

Jarde retains the two integer switches; its complete source recompiles under `--release 8`, and its five `-Xverify:all` output rows match the original (5/5 for this case and 11/11 across both Jarde negative cases). JADX also retains these switches in this case; its complete source recompiles and matches the original in all five rows. A fold that trusted only the integer-label-to-string comparisons and ignored `String.hashCode` would change the tested results, notably `Aa` to 10 and `abc` to 20. Require every literal's actual Java hash to equal its bucket key.

## Replay

Set `JARDE_CLI` to a built CLI, then run:

```sh
JARDE_CLI=/path/to/jarde-cli python3 replay.py
```

The script rebuilds and SHA-checks both Java 8 class files, captures `javap`, regenerates complete class source with JADX/Jarde, compiles the original and decompiler outputs where possible, and executes every compilable variant with `java -Xverify:all`. JADX's expected compilation failure for `ExtraHashUse` is asserted by the unresolved `r0` diagnostic. Jarde CLI used for this capture: SHA-256 `c54839b2a8cee1a300cb5911af2ca24b6a48f487f31f251a368b917d48407f86`; JADX 1.5.6. The temporary Cargo target used to build that CLI is removed after replay.

This evidence covers two negative cases for tasks 1.2/3.1 only; it does not complete string-switch recognition, its positive proof, or task 1.2 as a whole.

Root independently recompiled the frozen original, JADX and Jarde source files in a fresh temporary directory. The two original class SHA values above matched; original/Jarde matched 6/6 and 5/5 executable rows; WrongHashBucket's JADX matched 5/5; ExtraHashUse's JADX failed `javac --release 8` with unresolved `r0` as recorded. This independent replay validates the archived outputs; the script above additionally regenerates the Jarde/JADX outputs from the frozen inputs.
