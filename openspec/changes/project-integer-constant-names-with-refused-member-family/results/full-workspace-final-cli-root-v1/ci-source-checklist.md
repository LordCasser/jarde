# CF12 constant-name product CI checklist

The new product must bind the fresh complete workspace and final full-input CLI build to its exact Git bytes, and pass its own workflow jobs, steps and both seed inventories. The focused core test must occur exactly once. Old product CI is historical evidence only.

| Path | SHA-256 |
| --- | --- |
| .github/workflows/ci.yml | b545890a55cc4fda31d70b332889c111d789afbf0949581e96bf2b94ea6cd18a |
| Cargo.lock | 35cfeb18d481c129e849611f9eefbe931c3fec6df4a67a36e683fb072df77363 |
| Cargo.toml | 20eb9b1ee52f7229b1e6a8ceb0e4db39b7e888987bd157cc503cd02f3d9502e1 |
| crates/jarde-cli/Cargo.toml | b85621d77159a5106927808d76872812d22898e018749d49d606e2ecfe3d85c8 |
| crates/jarde-cli/src/export.rs | f2d7fa5439997e43cc4f5473d967c5fe1cd9975222f4e710674f9743a0223574 |
| crates/jarde-cli/src/main.rs | e33bbb15fe4154780a868f092e207cd14d3604da459a54fdfb0407afa2c19e63 |
| crates/jarde-cli/src/task.rs | 3e60775444304a0ecc870b269cc2afa0b7a1bacaf18c16e1fef6fe4e86e9061e |
| openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/capture/TestSwitchLabels.test/input/TestSwitchLabels$TestCls$Inner.class | 2d6221bb75ba81cba428c32e3139038982c4078c80a1af99532c3a45acd54a99 |
| openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/capture/TestSwitchLabels.test/input/TestSwitchLabels$TestCls.class | 4997ca33261d0585a1008dd860131c5ed3f2438206209bb7acb82c90eb743d22 |
| src/class_source.rs | bd797db1b47a0258c5031260f867fcd766ae9d5036eb4e8373f7be6fb827a71f |
| src/facade.rs | 9c55f3a53b0e435ea5c3e2bedac0fe66f32d474aecc9d1237029518f3703f22b |
| src/lib.rs | 11e6e9f4eb274bd794e23e85bc36b28e953e1888de05229802166f8586df4734 |
| tests/class_source.rs | bd78b01be60a958def040f5b934175c508a3512185a13f479490e46aa2ea9fc4 |
| tests/inner_class_static_mixed_folding.rs | 594ad71ad143550514629b90798eb8c0ac9aeaa4c09b7902ce8b3e0e6ed12232 |
| tests/member_class_static_folding.rs | fbbe15cd2db0f771eb9ced7a22ad292d7952ffe046e3930f65e1d89fe10a3b92 |
| tests/member_family_identity.rs | aa897f8b7cbe90a198c1ac5ff8baa448ca61fa638719df9319479494ddbd833d |
| tests/p3_nested_annotation_source.rs | e4263aed05c3f260e73a39fcb3939a3eb926a96f7e5ebd84966869a6b93a1f7f |
