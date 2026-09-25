# Floating annotation defaults: raw NaN-bit boundaries

This evidence starts from the existing 293-byte `FloatDefaults.class` produced by the checked-in Java source and `javac --release 8 -g:none`. It makes two one-constant-pool-payload patches, not source edits. Both patched classes remain Java 8 class files (major 52), keep the class/member tables byte-for-byte unchanged, and leave every non-target byte unchanged. `patch-manifest.json` records the constant-pool index, exact payload offset, raw bits, changed byte offsets, and SHA-256 for each patch.

- `positive-infinity-to-negative-qnan/FloatDefaults.class`: the F constant used by `positiveInfinity` changes from `0x7f800000` (+infinity) to `0xffc00000` (negative quiet NaN). Only bytes 181–182 differ; the other two bytes in the target four-byte payload already match.
- `canonical-nan-to-payload-nan/FloatDefaults.class`: the D constant used by `canonicalNaN` changes from `0x7ff8000000000000` to `0x7ff8000000000001`. Only byte 208 differs.

The reflection runner uses `floatToRawIntBits` and `doubleToRawLongBits`; these are the verified `java -Xverify:all` outputs:

```text
baseline:    80000000 / 1 / 7f800000 / 7ff8000000000000
F mutation:  80000000 / 1 / ffc00000 / 7ff8000000000000
D mutation:  80000000 / 1 / 7f800000 / 7ff8000000000001
```

Each patched input is decompiled with JADX 1.5.6 alongside the complete `FloatRunner` class, compiled with `--release 8`, and run. Compilation succeeds. For the F mutation, JADX emits `Float.NaN`, whose runtime raw bits are `7fc00000`, so this source output loses the patched sign bit. For the D mutation, JADX emits `Double.NaN`, whose runtime raw bits are the canonical `7ff8000000000000`, so this source output loses the payload bit. The generated source trees, javac logs, and runtime outputs are retained under `jadx-*` and the corresponding `*-jadx-*` files.

Jarde is also run over each complete input pair. Its generated annotation source omits all four float/double defaults; javac succeeds, then the unmodified reflection runner hits a `NullPointerException` on the first absent default. The complete generated class text and `--evidence all` JSON reports are saved as `*-jarde-FloatDefaults.*` and `*-jarde-FloatRunner.*`, with compiler/runtime logs and exit codes. This is the observed presentation gap, not an inference from the patched NaNs alone.

This evidence does **not** claim that all NaN values are inexpressible in Java source. The original annotation already expresses a canonical double NaN as `0.0d / 0.0d`; JADX's `Double.NaN` spelling recompiles to the canonical bits. The experiment establishes only that these two class-file bit patterns are not preserved by the specific JADX source output above. It does not exhaust possible Java constant-expression spellings for NaN signs or payloads.

The baseline annotation class SHA-256 is `c257ba990cb21bd1769ad7a949e65a8338934a3b7e202e0bc9ab964ab509ecad`. Full `javap -v -c -p` captures are included for the baseline and both patched annotation classes, plus both runner classes. The frozen Jarde CLI is `/tmp/jarde-cli-syntax-root-final`, SHA-256 `7a33dbed5b390009cb65802fd9264367e1d5854b9cc70dbced5ae1878109c822` (`jarde-cli-sha256.txt`).

## Replay

From this directory, run:

```sh
python3 run_raw_bit_boundaries.py
```

It rebuilds the baseline classes, applies the two constant-pool patches through `patch_raw_bits.py`, reruns JVM verification, `javap`, JADX, Jarde text/JSON generation, and the compile/runtime comparisons. The script has been verified from a copied directory. Set `JARDE_CLI` if the frozen CLI is elsewhere. Required tools are Python 3, javac, java, javap, JADX 1.5.6, and the specified Jarde CLI. No Cargo command is used.
