# Exact int-overload replay draft

Private v2, unrun collector and independent verifier for the existing 632-byte `CharProducerIntOverload.class` fixture. No repository files were modified. No Git, Cargo, JDK, javac, javap, or candidate CLI was executed while preparing these drafts. The collector records evidence only; its status never claims validation or acceptance.

The frozen original inputs are pinned by source SHA-256 `983fa4b49f8662f7c1736792ba1a840799e792dad36b6f705d00cd6ef43ccaf6` and class SHA-256 `18098c9d7257fabb8497107e42e86695be956dba666598ebd82daa2512722cd8`. The saved original run is exactly `46\n`. Its saved javap shows `render(Ljava/lang/String;)Ljava/lang/String;`, `String.charAt(I)C` at BCI 2, stores at 5 and 8, and `StringBuilder.append(I)Ljava/lang/StringBuilder;` at BCI 17. The fixture assigns `46` after the char-producing call, so the replay does not assume the producer alone determines the value.

The collector imports the SHA-pinned loop-latch v9 guarded command runner, configures its pinned JDK 23.0.1, and applies the 5 GiB free-space and 1 GiB target limits to each process. It writes under `/private/tmp`, adapting only the guard's stream-path formatter so absolute private raw paths are recorded. For `default` and `all`, it saves the exact complete class-source JSON stdout and full source, compiles with `-source 8 -target 8 -g:none -proc:none -Xlint:-options` and empty classpath/sourcepath directories, runs with `-Xverify:all`, and javaps both the frozen original and each fresh output class. Candidate BCI values are discovered from javap output and are not fixed in advance.

Collector invocation requires all future candidate/build identities as explicit values and a new output directory:

```text
python3 /private/tmp/jarde-typed-int-overload-replay-luna-v2/collect.py \
  --cli <frozen-cli-path> --cli-sha256 <sha256> \
  --metadata <candidate-metadata-path> --metadata-sha256 <sha256> \
  --build-execution <validation-execution-path> --build-sha256 <sha256> \
  --source-base <40-hex-source-base> \
  --out /private/tmp/jarde-typed-int-overload-replay-luna-v2/run-v2
```

After reviewing the execution record, root can invoke the independent verifier with the same candidate/build pins and a separate unused acceptance path:

```text
python3 /private/tmp/jarde-typed-int-overload-replay-luna-v2/verify.py \
  --execution /private/tmp/jarde-typed-int-overload-replay-luna-v2/run-v2/execution.json \
  --cli <frozen-cli-path> --cli-sha256 <sha256> \
  --metadata <candidate-metadata-path> --metadata-sha256 <sha256> \
  --build-execution <validation-execution-path> --build-sha256 <sha256> \
  --source-base <40-hex-source-base> \
  --acceptance /private/tmp/jarde-typed-int-overload-replay-luna-v2/acceptance-v2.json
```

The verifier independently binds the original command records/raw streams and JDK 23 tool hashes, checks every guarded command/stream, binds both reports to the exact original class bytes and exact method identity, and requires complete source-map ownership of all physical BCIs in `render`. It compares default/all complete text and source maps, compiles each complete report source, requires both verified runtimes to produce the exact original `46\n`, and checks fresh javap for `append(I)` with no `append(C)` in `render`. It writes an observations-only acceptance record; this one class replay does not establish the broader local-source-type change or CF12 completion.


The v2 collector checkpoints `execution.json` atomically after every command row is appended and after profile parsing. A guard/command exception records an interrupted row and retains prior command streams; a later collection error does not erase earlier evidence. The verifier additionally binds the source-map owner snapshot to `owner.location.snapshot`, checks exact raw method name/descriptor and member ordinal, and only writes its separate acceptance output after all checks pass.
