# Post-fix compound-lvalue replay

This replay uses the current Jarde CLI and writes only beneath this directory. It does not update
the frozen pre-change result in `../permanent-fixture-replay/evidence/`.

From the repository root, rebuild and replay with:

```sh
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo build -p jarde-cli
JARDE_COMPOUND_CLI=target/debug/jarde-cli \
  python3 openspec/evidence/java-syntax-2026-09-22/compound-assignments/post-fix-fixture-replay/replay_fixtures.py
```

The script also accepts `--cli PATH`, `--cli-sha256 HASH`, and `--evidence-dir PATH`; the matching
environment variables are `JARDE_COMPOUND_CLI`, `JARDE_COMPOUND_CLI_SHA256`, and
`JARDE_COMPOUND_EVIDENCE_DIR`. When a hash is supplied, the script checks it before and after the
replay. By default, evidence goes to this directory's `evidence/`, and the script rejects the old
pre-change evidence path.

The final independent root replay used CLI SHA-256
`7a33dbed5b390009cb65802fd9264367e1d5854b9cc70dbced5ae1878109c822`. The CLI hash matched before and after the replay. The whole class had 12
`Code` methods; original, JADX, and Jarde each compiled and ran under `java -Xverify:all`, and all
seven output lines matched:

```text
local=8:rhs=1
field=9:select=1:rhs=1
array=14:select=1:rhs=1
field-snapshot=9:select=1:rhs=1
array-snapshot=14:select=1:rhs=1
null=NPE:select=1:rhs=0
bounds=AIOOBE:select=1:rhs=0
```

The replay records 19 boundary modes: the 13 original source/identity/consumer/type/exception
cases and six verifier-safe ordering gaps. Original and JADX compiled and matched for all 19. Jarde
compiled and executed all 19; its two ordinary assignments, both snapshot updates, and the separate
null and bounds cases matched. It still differed on the five mismatched-identity/extra-consumer
cases, both `long` cases, and all six ordering-gap cases. These cases identify refusal boundaries,
but the current Jarde output is not a verified semantic fallback: it can compile and run while still
changing the original result. The tests establish that these candidates are not emitted as `+=` and
that their source anchors remain available; they do not establish behavior equivalence for those
out-of-scope methods.

Each added gap inserts an observable `rhs(0); pop` between a live left-side stack value and one of
the compound-update instructions: before the field `dup`, between `dup` and `getfield`, before
`putfield`, between array and index evaluation, between `dup2` and `iaload`, or before `iastore`.
All six patched classes verify and run; each original result includes two RHS calls. The Rust
recovery test requires these candidates to remain without `+=` and to retain the source anchors for
the receiver/array, read, RHS, final write, and inserted call. Full generated sources, `javap`,
compiler logs, execution outputs, and the checked CLI hash are in `evidence/summary.json` and its
per-case directories.
