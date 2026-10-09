# Candidate replay runner v1 plan

This runner lives in `openspec/changes/recover-covariant-child-array-initializers/results/`
and is prepared for root review; it has not been executed. It takes three
arguments: the future frozen CLI binary, that build's CLI metadata JSON, and a new
absolute output directory that must not already exist. The runner rejects a
relative output path before resolving it. The metadata's CLI path and binary SHA
must match the supplied executable, and `candidate_sources` must pin exactly
`init.rs`, `report.rs`, `build.rs`, and `Cargo.lock` with hashes matching the
workspace before any replay begins.

The fresh denominator is 24 candidate legs: the four factory/direct six-class
fixtures, all 18 frozen legacy matrix cases, and both numeric two-class cases. The
six-class direct family remains complete in each input jar and every class entry is
rendered. The historic 18-case count remains 16/18; its BigDecimal legs are the two
known failures and remain in the matrix. Factory's historical denominator is 2/2.
The direct six-class candidate's historic result is 0/2. These are historical
reference values, not assertions about the new candidate.

For each leg the runner verifies and copies the exact frozen input jar, enumerates
its complete `.class` entry set and hashes, invokes the supplied CLI once for every
class, saves each complete JSON report stream, and writes every returned Java source
under a private directory. It verifies that generated source paths equal the full
input class set. It then compiles all available generated sources together using the
frozen leg's compiler and flags, explicit empty classpath and sourcepath directories,
and a new output classes directory. A runtime is attempted only when the whole
source set rendered and compilation succeeded; it uses `-Xverify:all` and only that
candidate classes directory. The manifest records report markers, diagnostics,
method outcomes/quality, each `new` presentation record, source-map digest and BCI
anchors, candidate class set, actual argv/exits, tool binary hashes, and raw stdout/
stderr hashes and bytes.

Historical original streams are compared byte-for-byte without rerunning the
original programs. Factory raw streams come from the old `compose-constructed-reference-array-elements`
`legacy-family-cli2-root-v1` results. Direct raw streams come from the exact frozen
pathfix captures at `tests/fixtures/p3-heterogeneous-array-initializers-v3/direct/`
(`javac8/logs/direct_javac8_original-main.stdout` and the matching javac23 files);
each is 237 bytes with SHA-256
`fd3a20051507b79c6537ccc6fcc24a40fb1bb10e7fda4ea01ae06a08fdc0a71b`. Their byte
identities are recorded in this change's `results/baseline-reference-audit-v1.json`
(SHA-256 `2e2fa6698b2f52eb49761c7ba0b9bb523774048bf25fdaffb96c87b558226288`). The 14 ordinary legacy
matrix legs read their original captures from the permanent baseline tar archive;
the CT-frozen addendum legs reuse the archived CT original bytes after checking their
frozen original hashes are identical;
the BigDecimal originals come from `bigdecimal-historical-provenance-v1`; numeric
original streams come from `baseline-v1`. Before starting any candidate process,
the runner checks that all 24 frozen input jars and all 48 original stdout/stderr
captures exist and match their recorded hashes. It copies those original raw streams
into the new output, alongside all candidate command streams. Missing or changed
inputs/captures stop the run before candidate work begins.

The runner does not rerun JADX or the original programs. Its historical reference is
specific to this change's frozen six-class direct family: `baseline-reference-audit-v1.json`
and `baseline-root-verification-v2.json` (2,503 checks, passed). Those records pin
both JDK tool identities, all six original source/class hashes, the direct raw
original streams, old Jarde direct failures, and the already verified JADX direct
profiles (`--rename-flags none` 2/2; default 0/2). The runner rechecks the six-class
source/class and JDK hashes before replay. These are historical references only;
the prior source reports are not copied into fresh candidate cases.

For numeric legs, the runtime main class is read from the frozen numeric case
(`ConstructorPrimitiveConversionControls`) and verified against the input class
set. Method summaries retain descriptor, access flags, member index/declaration,
raw member identity, and body identity in addition to method outcomes and source
maps.

Every subprocess removes `JAVA_TOOL_OPTIONS`, `_JAVA_OPTIONS`, and
`JDK_JAVA_OPTIONS` from its explicitly supplied environment; it records whether
each variable was inherited on that call. This makes the JVM flags reproducible
without emitting any ambient option values into evidence.

The final manifest contains the runner and CLI/metadata hashes, historical manifest
identities, input jar entries, generated source identities, JDK executable hashes,
all commands and streams, and a closed list of every saved output file. `candidate`
success is calculated from actual full source coverage, absence of refusal/bytecode
stub markers, successful compilation and runtime, exact zero exit, and raw-stream
and exit equality. A completed replay may therefore contain failed candidate legs;
the runner reports them without removing them from the denominator.

root实际v1在任何候选进程前发现ct-legacy-frozen编译命令为addendum命名空间，原预检失败日志保留root-complete-candidate-replay-v1。原runner v1保留原SHA，root新建v2仅使用真实manifest里的addendum suffix；产品、输入、24腿分母均未改变。

root v2同样在候选进程前发现BigDecimal纠正腿也使用addendum命名空间，原失败保留。v3仅将该已存在family纳入相同精确label分支；root只读预检全部24腿的唯一编译命令成功，未放宽输入/原始流hash或完整源门槛。

root v3在原始schema的numeric case缺少family时预检KeyError（此前只读label核验临时补family，并非实际runner预检成功），原失败保留。v4明确numeric固定family用于label选择，保留其他真实schema字段；尚待实际回放。
