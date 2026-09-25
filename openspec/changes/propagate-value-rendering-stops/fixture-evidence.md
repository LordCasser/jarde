# Captured lambda fixture evidence

## Positive fixture

`tests/fixtures/p3-lambda-adaptation/v8/CapturedLambdaAdaptationProbe.class` is javac 8 classfile
version 52, 1,073 bytes, SHA-256
`e53c22eb8d0d06bc6bd2a71c8d3e2f8ff9523561e3d94cc9c4167adc537fa8b6`. It has four methods with
`Code` attributes, no exception handlers, one invokedynamic site, and one metafactory target.
Its `captured()` bytecode calls `producePrefix()` at BCI 0, stores the result at BCI 3, loads the
captured stack operand with `iload_0` at BCI 4, and executes invokedynamic at BCI 5 / CP #13.
These are three distinct origins: prefix producer BCI 0, capture operand producer BCI 4, and
lambda site BCI 5. The bootstrap's erased SAM descriptor is `(Object)int`; the implementation
handle targets the synthetic
`lambda$captured$0(int, String):int`; its instantiated SAM descriptor is `(String)int`. The
captured load and site have distinct BCIs. There is no `checkcast` in this class: the JVM applies
the required Object-to-String check from the real bootstrap descriptors.

The fixture test requests both essential and all evidence for `captured()`, checks equal method
text, and requires the dynamic String cast to map primarily to site BCI 5 / CP #13. The enclosing
lambda source range retains capture operand BCI 4 as a derived origin; BCI 4 is not the earlier
`producePrefix()` call at BCI 0. The cast has no checkcast instruction origin. A separate
`BoundNullLambdaAdaptationProbe` class records the creation-time null check and stays an explicit
refusal control.

That control is a 1,051-byte version-52 class with two methods carrying `Code`, no exception
handlers, one invokedynamic site, and one metafactory target; SHA-256 is
`938ffa55b73c1cb7bc20969af14faadaa14f78e81d108b25eddb5fae8776064d`.

The added corpus inputs and BLAKE3 hashes are recorded in
`tests/fixtures/corpus-fingerprint.json`. The reader's historical fixture census assertion was
left unchanged; it still needs its separate debt update before that census is expected to pass.

## Source and execution replay

The original four Java inputs compile with `javac --release 8 -g:none`. Running the original
classes with `java -Xverify:all` prints:

```text
35
31
java.lang.ClassCastException
boundNull:java.lang.NullPointerException
```

JADX 1.5.6 decompiles the four classes as a complete source set. The recovered source compiles
with `javac --release 8`, passes `java -Xverify:all`, and prints the same four lines. The positive
Jarde whole-class source also compiles with the fixture support, runner, and separate original
bound-null control source, passes `java -Xverify:all`, and prints the same output. Its
`captured()` method emits an explicit `(Object p0) -> ... (String) p0` adapter and retains the
captured value. In its all-evidence source map, that cast maps primarily to BCI 5 / CP #13, with
derived capture operand BCI 4.

The bound-null control is recovered separately. Its Jarde method outcome remains explanation-only
with fallback quality; its refusal diagnostic is `jre_lambda_sam_types` at the BCI 9 site and
says adapting the bound receiver would move the null failure from functional-value creation to
invocation. The class file's explicit `checkcast` at BCI 1 and `Objects.requireNonNull` at BCI 5
show the creation-time check before invokedynamic BCI 9 / CP #15. The original and JADX classes
throw `NullPointerException` while creating the functional value.

The CLI snapshot used for this replay was `/tmp/jarde-lambda-root-target/debug/jarde-cli`, SHA-256
`fbd01aebdb6093800f95ed6b52f576aa3d7a9b61a757021ae373b828b50b1d61`; it predates the active
budget-transport edits. The replay commands are:

```sh
javac --release 8 -g:none -d /tmp/captured-original \
  tests/fixtures/p3-lambda-adaptation/CapturedLambdaAdaptationProbe.java \
  tests/fixtures/p3-lambda-adaptation/BoundNullLambdaAdaptationProbe.java \
  tests/fixtures/p3-lambda-adaptation/LambdaCaptureAdaptationSupport.java \
  tests/fixtures/p3-lambda-adaptation/CapturedLambdaAdaptationRunner.java
java -Xverify:all -cp /tmp/captured-original CapturedLambdaAdaptationRunner
jadx -d /tmp/captured-jadx /tmp/captured-original/CapturedLambdaAdaptationProbe.class \
  /tmp/captured-original/BoundNullLambdaAdaptationProbe.class \
  /tmp/captured-original/LambdaCaptureAdaptationSupport.class \
  /tmp/captured-original/CapturedLambdaAdaptationRunner.class
javac --release 8 -d /tmp/captured-jadx-classes /tmp/captured-jadx/sources/defpackage/*.java
java -Xverify:all -cp /tmp/captured-jadx-classes defpackage.CapturedLambdaAdaptationRunner
```

The Jarde positive and negative class sources were each produced with `class-source`,
`--policy single-class --release 8 --evidence all`. The complete positive class source was then
compiled and run with the support, runner, and separate original bound-null control source. The
Jarde negative control remains refusal output and is not counted as a successful recovery. All
three runner invocations printed the same four lines shown above. In the Jarde invocation, the
first three lines exercise the recovered positive class; the last line exercises the separate
original bound-null refusal control. The permanent input SHA-256 values are:

```sh
/tmp/jarde-lambda-root-target/debug/jarde-cli class-source \
  --input /tmp/captured-original/CapturedLambdaAdaptationProbe.class \
  --class CapturedLambdaAdaptationProbe --policy single-class --release 8 \
  --format text --evidence all --output /tmp/jarde-CapturedLambdaAdaptationProbe.java
/tmp/jarde-lambda-root-target/debug/jarde-cli class-source \
  --input /tmp/captured-original/BoundNullLambdaAdaptationProbe.class \
  --class BoundNullLambdaAdaptationProbe --policy single-class --release 8 \
  --format text --evidence all --output /tmp/jarde-BoundNullLambdaAdaptationProbe.java
javac --release 8 -d /tmp/jarde-captured-classes \
  /tmp/jarde-CapturedLambdaAdaptationProbe.java \
  tests/fixtures/p3-lambda-adaptation/BoundNullLambdaAdaptationProbe.java \
  tests/fixtures/p3-lambda-adaptation/LambdaCaptureAdaptationSupport.java \
  tests/fixtures/p3-lambda-adaptation/CapturedLambdaAdaptationRunner.java
java -Xverify:all -cp /tmp/jarde-captured-classes CapturedLambdaAdaptationRunner
```

The positive source hash is recorded below; the negative bound-null file remains intentionally
refusal output. Input and generated positive source hashes are:

| Input | SHA-256 |
| --- | --- |
| `CapturedLambdaAdaptationProbe.java` | `2cce01d7801b0c7a5042fb4e4e30df451d095e7a6be10a63a976da7dd309e4dd` |
| `CapturedLambdaAdaptationRunner.java` | `f873a677fc04903d6ef8792d66dd4d414503faa641ee859bceef33087fcd9e99` |
| `LambdaCaptureAdaptationSupport.java` | `be5ee29ebf0005d92af7b24eceaebcdbca8ac8cc9c3f48469bbecb6109d8319f` |
| `BoundNullLambdaAdaptationProbe.java` | `20bf874bc8ed913845fb051593538527d4485deecd742d5c93b244370cf5f90b` |
| `CapturedLambdaAdaptationProbe.class` | `e53c22eb8d0d06bc6bd2a71c8d3e2f8ff9523561e3d94cc9c4167adc537fa8b6` |
| `BoundNullLambdaAdaptationProbe.class` | `938ffa55b73c1cb7bc20969af14faadaa14f78e81d108b25eddb5fae8776064d` |
| JADX generated `BoundNullLambdaAdaptationProbe.java` | `27c6b92b9e51e4527ceec0c636462532e40969e8174c63c29363b71f6b8b5c58` |
| JADX generated `CapturedLambdaAdaptationProbe.java` | `1ced6d62ab06426ddbeb06e08c5f2ca3a665a94610e3233203eb7b4fce495987` |
| Jarde generated `BoundNullLambdaAdaptationProbe.java` | `d71dda5f479ece3e80f94c9e5ad7335a07d6297b6fde9c05154759294ffa758a` |
| Jarde generated `CapturedLambdaAdaptationProbe.java` | `3ec8aea347ca7a322422e7fb430c3c47423728390c0e1a08bb551037eebe2f85` |

`javac` was 23.0.1 targeting release 8; JADX was 1.5.6. `tests/p3_lambda_adaptation.rs`
contains permanent essential/all, source-origin, refusal-code, internal budget and JDK execution
assertions. Root ran all 11 tests, including both ignored JDK tests, against the active production
edits; all passed.
