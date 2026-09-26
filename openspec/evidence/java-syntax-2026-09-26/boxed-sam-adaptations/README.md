# Boxed SAM adaptation baseline

`BoxedSamProbe.java` is a self-written Java 8 fixture. The frozen class was compiled on OpenJDK 23.0.1 with `javac --release 8 -g:none`; `source.sha256` and `original-class.sha256` record the inputs. `v8/BoxedSamProbe.class` is the only retained binary. `javap.txt` is its complete `javap -v -c -p` listing.

The fixture covers `Supplier<Integer>` backed by `int supply()`, `Function<String,Integer>` backed by `int staticRef(String)`, two pairs of method-reference sites in `chainedSites()`, and `Function<Integer,int[]>` backed by `int[]::new`. The classfile declares class version 52. Its LambdaMetafactory `metafactory` arguments are:

| BSM | erased SAM | implementation handle | instantiated type |
| --- | --- | --- | --- |
| 0 | `()Object` | `REF_invokeSpecial BoxedSamProbe.supply:()I` | `()Integer` |
| 1 | `()Object` | `REF_invokeSpecial BoxedSamProbe.supplyMinimum:()I` | `()Integer` |
| 2 | `(Object)Object` | `REF_invokeStatic BoxedSamProbe.staticRef:(String)I` | `(String)Integer` |
| 3 | `(Object)Object` | `REF_invokeStatic BoxedSamProbe.lambda$arrayCtor$0:(I)[I` | `(Integer)[I` |
| 4 | `()Object` | `REF_invokeSpecial BoxedSamProbe.supplyOther:()I` | `()Integer` |
| 5 | `(Object)Object` | `REF_invokeStatic BoxedSamProbe.staticRefOther:(String)I` | `(String)Integer` |

The synthetic array helper has descriptor `(I)[I`, is `private static synthetic`, and its complete Code is in `javap.txt`: `0: iload_0; 1: newarray int; 3: areturn` (`stack=1`, `locals=1`). `chainedSites()` invokes dynamic sites at BCIs 2, 7, 17, and 22, joining two `dispatch` calls. The normal SAM outputs and the null-unboxing and negative-array-size outcomes are frozen in `original-runtime.txt`.

`jadx.java.txt` and `jadx-runtime.txt` are from JADX 1.5.6. JADX expands the array constructor reference into an `i -> { return new int[i]; }` lambda; its compiled runtime output matches the original (`runtime-comparison.txt`). Local JADX source checkout: `/Users/lordcasser/workspace/testzone/jadx`, revision `2fb1b16386941660fda07e9017285aec40fcb37f`; relevant visitor is `jadx-core/src/main/java/jadx/core/dex/instructions/invokedynamic/CustomLambdaCall.java` (it accepts LambdaMetafactory `metafactory`/`altMetafactory`, reads implementation handle and instantiated method type, and projects the implementation method). `InvokeCustomBuilder.java` handles invokedynamic construction. The installed 1.5.6 binary, not a locally rebuilt checkout, produced this output.

`jarde.java.txt` is the complete current pre-fix Jarde class-source output from a single-class request; `jarde-summary.txt` extracts the five relevant member outcomes. The lambda sites are refused and their enclosing methods retain whole-method bytecode markers. The array diagnostic identifies the `(Integer)[I` versus `(I)[I` input mismatch. The Jarde text does not compile as Java: `jarde-javac.status` is `1`, and the compiler diagnostics are saved in `jarde-javac.stderr`. Its generic-signature projection diagnostics are a separate non-goal recorded in the design.

## Replay

From the repository root, run:

```sh
sh openspec/evidence/java-syntax-2026-09-26/boxed-sam-adaptations/replay.sh
```

The script recompiles the fixture using `javac --release 8 -g:none`, runs the original and JADX-reconstructed classes with `java -Xverify:all`, captures `javap`, and regenerates the Jarde source and selected report rows. It needs `javac`, `java`, `javap`, `shasum`, `python3`, and JADX 1.5.6 on `PATH`. Set `JARDE_CLI=/path/to/jarde-cli` to select a Jarde executable; otherwise it uses `/tmp/jarde-cli-audit-baseline`. This frozen pre-fix executable has SHA-256 `60c045b8f94d363950525692f5ce9e5f5cde6f78134806d75801cd0cae89a573` (`jarde-cli.sha256`). It is outside the repository and is not rebuilt by the replay script, so reproducing the Jarde side on another machine requires an equivalent pre-fix CLI. If it is missing, the script records `unavailable` in `jarde.status` and still performs the javac/JADX parts when JADX is installed. Temporary decompiler and compiler directories are removed on exit.

The source-level visitor comparison is an inspection of the local JADX checkout; no second JADX build was made. The snapshot is a baseline, not a claim that Jarde's output is executable or behaviorally equivalent. `java -Xverify:all` executed the original and JADX reconstruction; Jarde execution was unavailable because its emitted class source does not compile.

## Root acceptance

On 2026-09-26 root independently ran `replay.sh`, confirmed the source SHA-256 `a9fa73d73ac3ea05eeccd19a80042a604376750e528579f3296aad3818a528b6`, class SHA-256 `42dec3fe7f959bf4577417bd497cc14b5a55c59b0e6354e8eb65a9be33e8d32b`, matching original/JADX verifier-checked runtime files, and baseline Jarde javac status `1`. The classfile's six bootstrap argument triples and exact synthetic array-helper Code were checked against `javap.txt`. `openspec validate recover-boxed-sam-adaptations --strict` passed. This accepts the frozen gap only; tasks 2.x–3.x remain open.
