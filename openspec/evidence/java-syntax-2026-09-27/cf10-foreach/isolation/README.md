# CF-10 failure isolation

This report narrows two failures in the existing [CF-10 replay](../README.md). It does not establish coverage of the full fixed JADX unit. The frozen negative test contains eight loop shapes and disables compilation; its only assertion is that output contains no `:`. In particular, that assertion cannot show that the stride-index method was retained.

Environment: `javac 23.0.1` with `--release 8 -g -Xlint:-options`, JADX `1.5.6` from the frozen checkout at `2fb1b16386941660fda07e9017285aec40fcb37f`, and the Jarde CLI built from commit `d73aa754eb56ac93e02eacc7e7bc05ebb93d933f`. Both inputs below are complete, single-class Java sources. Their exact source and class SHA-256 values are listed below.

## Step-index negative slice

[`StepIndex.java`](step-index/original/StepIndex.java) contains only a zero-based `i += 2` array loop, its return, and a runner. It is the smallest distinct shape from the frozen negative fixture that preserves the step-index behavior without bundling other negative loops.

| Artifact | SHA-256 |
|---|---|
| Input source | `874313396926ffdc2593212edc1215e1ab7580707f277f6382c4712cc69ea592` |
| Input class | `64b5baa6270ce4604c0a70fa0a6d3438902bdeaa9f3bc3f7cb5105b40c512344` |
| JADX source | `13c2d06f4731c0aa452822aa1a11b775d06050883f012fb8c764a5ed02d1349b` |
| Jarde source | `37ae8a2bc053dbd22486266b29de686777eb21fb459ea5df29b3db2e9e514137` |

The original and JADX full source each compile under Java 8 and run with `-Xverify:all`, both printing `4`. JADX retains `for (int i = 0; i < values.length; i += 2)`. Jarde does not convert the stride loop to foreach; its `everyOther` body contains only an explanation that `local 1 crosses a quoted fallback region`. The complete Jarde source fails Java 8 compilation with a missing return in that method. This locates the observed failure at local definition/use presentation across a quoted region. The loop's non-foreach shape itself is retained by JADX and is not evidence of a CF-10 foreach-conversion mismatch. The internal Region failure path is still not traced.

## List-to-Iterable call slice

[`ListToIterable.java`](list-iterable-call/original/ListToIterable.java) has one `consume(Iterable)` helper that prints `called` and one `main` call with the result of `Arrays.asList`. There is no loop, foreach, overload, or parameterized helper declaration.

| Artifact | SHA-256 |
|---|---|
| Input source | `a6880b2a064ec1390bca8245101c3bcdf0ac58d01b69cc0c5d0f4ce69eedb274` |
| Input class | `bbae590902f24b48fa064c25b59247ec71fcb1534ec0818a54ce437b5f1ba0ce` |
| JADX source | `9ac68d263a1ef3e8e0af896fdba0161982de5989d410a51d30b8bb23c8ef0e62` |
| Jarde source | `32d7fa4b815ec49d5c9d9c6ea3d8a1087a1e910325f1664e6d736f78739dc205` |

The original, JADX and Jarde complete source all compile with `javac --release 8`. The original and JADX run with `java -Xverify:all` and print `called`; Jarde compiles but prints nothing. Jarde quotes the complete call at BCI 22 because the argument is presented as `java.util.List`, the descriptor requires `java.lang.Iterable`, and `invocation_argument` has no safe reference-conversion evidence. Since this failure survives without a loop or overload, it is a distinct call-argument typing gap. The narrow proposal is [preserve-list-iterable-invocation-widening](../../../../changes/preserve-list-iterable-invocation-widening/proposal.md).

## Full CF-10 fixture

The original full `ForeachCases.java` source SHA-256 is `2e082a247e01798f1f8172d0e8ebcbc6d3fb24224366ae9d77d3887d2d4b7eb5`; its Java 8 compiled input class SHA-256 is `6b65ab7efe39ce153eb008f626ed2f0cd382b2260c3c10a241fef8031b0c6864`. Original and JADX compile and run with output `10`, `abc`, `4`. Jarde recovers the two foreach methods, while `everyOther` independently fails compilation as in the step slice and `main` independently quotes the List-to-Iterable call as in the call slice. Thus the two diagnostics in the full class correspond to separate recoverability areas; fixing either one does not make the other pass.

The reproducible source-generation command for each class is:

```sh
CARGO_TARGET_DIR=/tmp/jarde-cf10-target CARGO_INCREMENTAL=0 cargo run -q -p jarde-cli -- class-source --input INPUT.class --class CLASS --policy single-class --release 8 --format text > Jarde.java 2> Jarde.report
```

From this report's directory, the focused replays are:

```sh
mkdir -p /tmp/cf10-step/original /tmp/cf10-step/jadx-classes /tmp/cf10-step/jarde-classes
javac --release 8 -g -Xlint:-options -d /tmp/cf10-step/original step-index/original/StepIndex.java
jadx -d /tmp/cf10-step/jadx /tmp/cf10-step/original/StepIndex.class
javac --release 8 -g -Xlint:-options -d /tmp/cf10-step/jadx-classes /tmp/cf10-step/jadx/sources/defpackage/StepIndex.java
java -Xverify:all -cp /tmp/cf10-step/original StepIndex
java -Xverify:all -cp /tmp/cf10-step/jadx-classes defpackage.StepIndex
javac --release 8 -g -Xlint:-options -d /tmp/cf10-step/jarde-classes step-index/jarde/StepIndex.java # expected failure: missing return

mkdir -p /tmp/cf10-list/original /tmp/cf10-list/jadx-classes /tmp/cf10-list/jarde-classes
javac --release 8 -g -Xlint:-options -d /tmp/cf10-list/original list-iterable-call/original/ListToIterable.java
jadx -d /tmp/cf10-list/jadx /tmp/cf10-list/original/ListToIterable.class
javac --release 8 -g -Xlint:-options -d /tmp/cf10-list/jadx-classes /tmp/cf10-list/jadx/sources/defpackage/ListToIterable.java
javac --release 8 -g -Xlint:-options -d /tmp/cf10-list/jarde-classes list-iterable-call/jarde/ListToIterable.java
java -Xverify:all -cp /tmp/cf10-list/original ListToIterable
java -Xverify:all -cp /tmp/cf10-list/jadx-classes defpackage.ListToIterable
java -Xverify:all -cp /tmp/cf10-list/jarde-classes ListToIterable
```

For every successful compile, the runner output is shown in the respective `*.out` artifact. The frozen JADX negative assertion and recognizer evidence remain summarized in the parent README; one isolated probe is not evidence that every array and Iterable loop case in CF-10 is covered.
