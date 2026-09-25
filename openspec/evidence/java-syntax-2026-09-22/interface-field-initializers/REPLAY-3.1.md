# Task 3.1 replay entry

`replay_3_1.py` is the post-change replay entry. It writes only to a new output directory and
leaves the saved 2026-09-22 audit outputs untouched. It checks each frozen classfile SHA before
running, compiles the saved original source, stored full JADX source, and current Jarde output with
`javac --release 8`, then runs every successfully compiled variant with `java -Xverify:all`. The
original-source compile supplies the helper classes; the replay then places the exact frozen input
class under test on the runtime class path. This preserves the intentionally patched classfile
observations in the refusal and phase-boundary cases.

After the root task builds the current CLI, run from the repository root into a fresh directory:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/replay_3_1.py \
  --cli /absolute/path/to/current/jarde-cli \
  --out /tmp/jarde-interface-init-3-1-replay
```

`summary.json` records each fixture SHA, hashes for its Java source inputs, the current CLI SHA,
compiler and JVM statuses, runtime output, and two CLI budget runs. Cases cover the ordinary valid
interface, the field-table permutation, extra effect, duplicate/missing write, branch, qualified
forward read, exception-handler edge, and `ConstantValue` phase boundary. A failed Jarde compile is
an observed refusal; the replay records no runtime result for that source. The field-table and
forward-read positives should compile and retain the original runtime traces. The phase-boundary
Jarde source must not change the frozen original `0|9` observation into a claimed successful
`9|9` recovery.

The two budget runs use CLI-supported `method_bodies=1` and `output_bytes=1024`; their process
status and any emitted text are preserved and compilation/runtime are attempted only if the CLI
returned source. The CLI accepts neither an IR-item budget nor a cancellation token. Cancellation is
therefore recorded as library-only and is covered by
`tests/interface_initializer_projection.rs::cancellation_before_projection_publishes_no_class_source_report`;
the CLI replay does not fabricate a cancelled report or label an unrun class as recovered. This
adapter limit must remain explicit in the 3.1 acceptance record.

The frozen class inputs and SHA-256 values checked by the script are:

| Case | Class input | SHA-256 |
| --- | --- | --- |
| normal | `original/InterfaceInitProbe.class` | `e04abe561a505d9022039776b8b29de22e35f6d4dba45a08f19a9b5c07afb527` |
| field table reordered | `field-table-reordered/InterfaceInitProbe.class` | `01f953662dc388e8682740967e007bfeaa5671576329f87a75b0989254470fb6` |
| extra effect | `boundaries/extra-effect/BoundaryProbe.class` | `ddbeb07c02d68beddc307e70684cb9c672cf7ffa47ff4ec86719bef6467ebeac` |
| duplicate/missing write | `boundaries/duplicate-write/BoundaryProbe.class` | `2c8ce2e3a9fc416b80ef5151ef185b4b55053163c860ca4bba75addcc84e1cd8` |
| branch | `boundaries/branch/BoundaryProbe.class` | `c61471f60100781f462145d958428840c730e7c98cb7a23e2c77c45b554cd254` |
| forward binding | `forward-binding-exception-edge/forward-binding/ForwardProbe.class` | `916d57e880cd8bd99c5c705fab87dea60d41365afa3fee3604e5d2dd3a8f7b29` |
| exception handler | `forward-binding-exception-edge/exception-handler/ExceptionProbe.class` | `6a7ee16469492c295c64fcbaaedd506c6ab322865b4f03dd62561b0d7619872a` |
| constant phase boundary | `constant-phase-boundary/PhaseProbe.class` | `e9e686a011b047f24d352396ec5a3ccc20d8a502c6515ae063b6f0a76b569f5b` |
