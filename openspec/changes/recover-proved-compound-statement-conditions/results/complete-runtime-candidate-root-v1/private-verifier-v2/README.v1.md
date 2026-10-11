# Complete-class runtime verifier draft

`verify.py` is a read-only verifier for the new full-class runtime `execution.json`. It independently checks the recorded compile and runtime commands, both JDK legs, the 65-row matrix, output class sets, Jarde default/all source and maps, the JADX observer import adaptation, helper/tool/fixture pins, and all raw streams. Its pass state depends on those individual observations, not the runner's summary status.

The verifier does not launch Java, `javac`, JADX, or the Jarde CLI. It checks the original compiled class against the frozen 2,848-byte SHA-256 pin as well as checking the input source. Root must provide the actual execution record, fresh CLI SHA-256, and a new output path when the run is complete:

```text
python3 verify.py --execution <execution.json> --out <new-verification.json> --cli-sha256 <fresh-cli-sha256>
```

The expected CLI path is fixed to `/private/tmp/jarde-compound-condition-candidate-cli-root-v1`; the SHA is required as an explicit argument because root has not supplied its final pin yet. The fixture, matrix, shared plan, and source pins are frozen in the script.

The verifier intentionally does not assert a bytecode instruction count or independently cross-check source-map BCIs against `javap`; root's public source-origin tests cover those facts. It expects eight successful target compiles, eight runner compiles, and eight runtime commands across two JDKs. Original and Jarde default/all output must match all 65 expected rows; JADX must differ only at the recorded `FLOAT-nan` row.

This is an unexecuted private draft. No Python interpreter, JDK, CLI, Cargo, or Git command was run against it. Root should review the exact new execution schema and paths before using it.
