# SequentialLoops2 private collector preparation

This preparation targets the single frozen class `cf11.UpstreamSequentialLoops2` plus `cf11.Runner`, using inputs from `/private/tmp/jarde-cf11-sequential-loops2-input-luna-v2`. It reuses the root-v3 controls collector's dynamic `target_classes` manifest, dual-JDK compile/run, per-target Javap, exact Java 23 target-class archive, two JADX profiles, and per-class Jarde default/all renders. The only collector-specific change is naming the generated archive `sequential-loops2.jar`; the root-v3 collector already imports `re` at module scope and has no local-import scope trap.

`inputs_manifest.json` binds source SHA-256 and byte lengths independently for `UpstreamSequentialLoops2.java` and `Runner.java`. The hashes are source pins. They are not the decompiler's class digest. The checker preserves the root-v5 policy: it checks the rendered class digest's BLAKE3 shape and location/owner/default-all consistency, while physical `.class` files are pinned separately with SHA-256. It does not claim to recompute the rendered BLAKE3 digest.

`check_sequential_loops2.py` is based on the root-v5 observation checker. Its narrow Javap adjustment recognizes the `static {};` declaration immediately preceding `descriptor: ()V` as `<clinit>()V`, while retaining name/descriptor matching for ordinary methods and constructors. This fixture has no switch instructions; the inherited numeric Javap extractor rejects `tableswitch` and `lookupswitch` so table labels cannot be mistaken for instruction offsets. The checker reports method-level missing physical BCI origins as quality gaps and preserves runtime differences and compile failures as observations. A captured observation is not product or behavioral acceptance.

Invocation template; root must supply the frozen CLI, receipt, JDK manifest, JADX, rootguard, scanner, and classfile-parser paths and hashes at run time:

```sh
python3 /private/tmp/jarde-sequential-loops2-collector-prep-luna-v1/collect_sequential_loops2.py \
  --inputs-dir /private/tmp/jarde-cf11-sequential-loops2-input-luna-v2 \
  --inputs-manifest /private/tmp/jarde-sequential-loops2-collector-prep-luna-v1/inputs_manifest.json \
  --out NEW_CAPTURE_DIR --cli FROZEN_CLI --cli-sha256 CLI_SHA256 \
  --build-receipt BUILD_RECEIPT --build-receipt-sha256 RECEIPT_SHA256 \
  --jdk-manifest JDK_MANIFEST --jdk-manifest-sha256 JDK_MANIFEST_SHA256 \
  --jadx JADX --jadx-sha256 JADX_SHA256 --jadx-jar JADX_JAR --jadx-jar-sha256 JADX_JAR_SHA256 \
  --guard ROOTGUARD --guard-sha256 ROOTGUARD_SHA256 \
  --scanner TARGET_SCANNER --scanner-sha256 SCANNER_SHA256 \
  --classfile-parser CLASSFILE_PARSER --classfile-parser-sha256 PARSER_SHA256
python3 /private/tmp/jarde-sequential-loops2-collector-prep-luna-v1/check_sequential_loops2.py \
  --execution NEW_CAPTURE_DIR/execution.json --out NEW_VERIFICATION.json
```

Preparation was limited to source inspection, JSON/source pin creation, AST parsing, and static review. No collector, checker, Java, JADX, CLI, build, Git, or shared-workspace action was run.
