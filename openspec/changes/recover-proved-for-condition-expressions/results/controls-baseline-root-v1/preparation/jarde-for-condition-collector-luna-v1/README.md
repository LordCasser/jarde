# CF11 full-source observation collector (prepared only)

This directory contains a collector and an independent checker for frozen Java 8 CF11 class-source observations. They have **not** been executed. The collector writes a raw command ledger, exact stdout/stderr streams, frozen input and tool pins, class inventories, JADX profiles, and Jarde default/all full-class reports. The checker reads that capture and independently reconstructs the command policy, inputs, class inventories, method metadata, `Code` envelopes, Javap instruction offsets, and every physical-BCI source map.

The manifest keeps the target set dynamic. It must be a root-frozen JSON document with this shape (replace the illustrative values with exact source paths, sizes, and SHA-256 values):

```json
{
  "package": "cf11",
  "source_level": "Java 8",
  "target_classes": [
    {"class": "cf11.ForConditionEffects", "source": "ForConditionEffects.java", "bytes": 0, "sha256": "..."},
    {"class": "cf11.ForConditionReads", "source": "ForConditionReads.java", "bytes": 0, "sha256": "..."},
    {"class": "cf11.ForConditionRefusals", "source": "ForConditionRefusals.java", "bytes": 0, "sha256": "..."}
  ],
  "runner": {"class": "cf11.Runner", "source": "Runner.java", "bytes": 0, "sha256": "..."}
}
```

`target_classes` may contain one or more unique `cf11` top-level classes; the source filenames must match their class names. The checker expects exactly those target `.class` files plus `cf11/Runner.class`, with no anonymous or extra generated classes. Each original set is compiled with JDK 8 and JDK 23 using annotation processing disabled, debug metadata enabled, and empty class/source paths, then run with `-Xverify:all`. Javap is captured for every target on both JDKs. JADX receives a stored ZIP containing precisely the JDK 23 originals and is run in the default and rename-none profiles. Each Jarde source is captured independently in default and all-evidence mode; the complete generated sources are compiled together with Runner. If that compile fails, each generated target is also tried with the other original sources and Runner, preserving the failed combined compile and all isolated observations without editing generated text.

The invocation below documents required inputs; the root supplies every path and hash at run time. The build receipt must bind the selected frozen executable with mode `0o555` and status `passed_candidate_build_only`. The executable and receipt are separate pinned inputs. A baseline or later postfix capture therefore uses the CLI and receipt selected by root without embedding a build number or filesystem path in this preparation.

```sh
python3 /private/tmp/jarde-for-condition-collector-luna-v1/collect_controls.py \
  --inputs-dir ROOT_FROZEN_INPUTS_DIR --inputs-manifest ROOT_FROZEN_MANIFEST \
  --out NEW_CAPTURE_DIR --cli FROZEN_CLI --cli-sha256 CLI_SHA256 \
  --build-receipt BUILD_RECEIPT --build-receipt-sha256 RECEIPT_SHA256 \
  --jdk-manifest JDK_MANIFEST --jdk-manifest-sha256 JDK_MANIFEST_SHA256 \
  --jadx JADX --jadx-sha256 JADX_SHA256 --jadx-jar JADX_JAR --jadx-jar-sha256 JADX_JAR_SHA256 \
  --guard ROOTGUARD --guard-sha256 ROOTGUARD_SHA256 \
  --scanner TARGET_SCANNER --scanner-sha256 SCANNER_SHA256 \
  --classfile-parser CLASSFILE_PARSER --classfile-parser-sha256 PARSER_SHA256
python3 /private/tmp/jarde-for-condition-collector-luna-v1/check_controls.py \
  --execution NEW_CAPTURE_DIR/execution.json --out NEW_VERIFICATION.json
```

The default receipt binding fields are `candidate_cli`, top-level `status`, status value `passed_candidate_build_only`, and binding field `mode`. If the root's receipt schema differs, pass the confirmed `--receipt-cli-object`, `--receipt-status-key`, `--receipt-accepted-value`, and `--receipt-mode-key` values. No receipt path, CLI hash, JDK/JADX hash, method descriptor, success-command count, or expected runtime output is frozen in these scripts.

The collector applies the existing rootguard/scanner pattern: retain at least 5 GiB free, stop if target growth exceeds 1 GiB, sample every second, and keep each process in its guarded process group. It treats compile and tool failures as observations. The checker requires the original compile/run oracles to succeed and agree across JDKs; a failed generated compile has no claimed runtime. Successful generated runs retain exact comparison results against the same-JDK original oracle. JADX output is assessed independently from Jarde output.

Preparation checks performed: both Python files were parsed with `ast.parse` without writing bytecode caches. No collector/checker command, Java/JADX/CLI command, project build, shared workspace write, Cargo/JDK/JADX setup, or Atlas indexing was performed.
