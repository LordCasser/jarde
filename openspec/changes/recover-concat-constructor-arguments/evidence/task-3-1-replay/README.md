# Task 3.1 replay

The replay uses the frozen Java 8 input and frozen JADX source from
`openspec/evidence/java-syntax-2026-09-26/exception-constructor-concat/`. The Jarde
source below was emitted directly by the rebuilt CLI; it was not edited.

Run the build from the repository root, then change to this directory for the replay:

```sh
CARGO_TARGET_DIR=/tmp/jarde-concat-constructor-agent-target cargo build --locked -p jarde-cli
cd openspec/changes/recover-concat-constructor-arguments/evidence/task-3-1-replay
javac --release 8 -g:none -Xlint:-options -d original-classes original-src/Probe.java
javac --release 8 -g:none -Xlint:-options -d jadx-classes jadx-src/Probe.java
jar cf input.jar -C original-classes .
java -Xverify:all -cp original-classes Probe > original-run.txt
java -Xverify:all -cp jadx-classes Probe > jadx-run.txt
/tmp/jarde-concat-constructor-agent-target/debug/jarde-cli class-source --evidence all --input input.jar --policy plain-jar --class Probe > Probe.java 2> jarde.json
/tmp/jarde-concat-constructor-agent-target/debug/jarde-cli class-source --input input.jar --policy plain-jar --class Probe > Probe-default.java 2> jarde-default.json
javac --release 8 -g:none -Xlint:-options -d jarde-classes Probe.java
java -Xverify:all -cp jarde-classes Probe > jarde-run.txt
cmp original-run.txt jadx-run.txt
cmp original-run.txt jarde-run.txt
cmp Probe.java Probe-default.java
```

All three executions print five lines, each `arm-2`. The original, JADX-recompiled,
and Jarde-recompiled `Probe.class` files have the same SHA-256:
`d28a756995fba8e47b2b57ded7f47c9d240a5568184b8c52445dfc81a8905289`.
The generated source contains both `throw new java.lang.ArithmeticException("arm-" + arg0);`
and `return new java.lang.ArithmeticException("arm-" + arg0);`. Default and `all`
CLI text are byte-identical; the all-evidence report is retained in `jarde.json`.
