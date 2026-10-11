# CF11 outer-assignment reader census prep

This is a private static preparation package. It does not modify the repository, copy the fixture, regenerate `corpus-fingerprint.json`, run Cargo/JDK/Jarde/shared, or mutate Git.

The pinned javac 8 classfile from baseline capture has SHA-256 `28b92d58ce18fb08362bd39b0558de98c3c41830cc8cbc6bd60ec3a183b21956`. Its classfile methods were enumerated and their `Code` bytes passed through the existing pure-Python instruction decoder pinned in `evidence.json`. Branch and switch target rows use the same record definition as `MethodCodeFacts::control_flow_targets`; exception-table handlers are reported separately and excluded from the reader test's `branch_targets` accumulator. The helper also checks every branch/switch target lands on a decoded instruction start.

Measured rows are in `census.json`: `<init>()V` contributes one Code body; `positive`, `noCastAssign`, `bodyEffectBeforeAssign`, and `extraNextConsumption` contribute two direct branch targets apiece. All five methods have zero handlers and no jsr/jsr_w. Therefore the reader population delta is `(1 class, 5 bodies, 0 handler records, 8 branch/switch targets, 0 subroutines)`, and the proposed assertion changes from `(1104, 4914, 469, 2992, 10)` to `(1105, 4919, 469, 3000, 10)`.

`reader-census.patch` is the minimal private source diff: a three-line measured ledger entry and the tuple update in `repository_class_fixtures_validate_without_false_target_rejections`. It assumes root copies this exact class under `tests/fixtures/p3-iterator-assignment`; the test recursively counts classfiles and needs no fixture-list edit. The source Java/Runner, fixture placement, regenerated corpus fingerprint, and real acceptance remain root work. This prep is not an acceptance result.
