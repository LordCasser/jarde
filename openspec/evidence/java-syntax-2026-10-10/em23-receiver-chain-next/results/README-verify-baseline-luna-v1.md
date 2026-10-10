# EM23 receiver-chain baseline verifier

`verify-baseline-luna-v1.py` independently authenticates the closed EM23 `baseline-root-v2` evidence. It does not invoke Java, JADX, Jarde, or the collector. It checks the pinned manifest/inventory, all 33 command records and streams, the fixed JDK/JADX/Jarde binaries and CLI metadata, the two-class JADX input JAR, and all original/JADX whole-source compile and `-Xverify:all` runtime legs.

The verifier also reads both original `javap -p -c -s -v` reports and checks root and nested-class field/method identities, access flags, archive entry names and ordinals, BLAKE3 owner identities, and source-map primary/derived BCI coverage. It requires default/all root and child text plus per-method maps to match. The four Jarde compilation failures remain explicit: `test2(I)V` contains the `jarde_refused_body()` explanation, and javac reports that symbol unresolved. They are recorded as a product gap; the verifier expects no Jarde runtime legs.

Run with the pinned BLAKE3 Python dependency after reviewing the prepared script:

```sh
uv run --no-project --with blake3==1.0.11 python -B \
  openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/results/verify-baseline-luna-v1.py
```

On acceptance it writes `results/baseline-independent-acceptance-luna-v1.json` and refuses to overwrite an existing file. Preparation and syntax checking do not mean the verifier has accepted the evidence.
