# 产品自身CI源码核对表

源码表仅列关键入口/回归；实际验收还须重建并绑定完整4700输入到产品Git/live/最终CLI。自身CI未执行，不能借此前产品CI。

| Path | SHA-256 |
| --- | --- |
| .github/workflows/ci.yml | b545890a55cc4fda31d70b332889c111d789afbf0949581e96bf2b94ea6cd18a |
| Cargo.toml | 20eb9b1ee52f7229b1e6a8ceb0e4db39b7e888987bd157cc503cd02f3d9502e1 |
| Cargo.lock | 35cfeb18d481c129e849611f9eefbe931c3fec6df4a67a36e683fb072df77363 |
| crates/jarde-java/src/region.rs | b1330dfab9e71076f72e7ed5d0dfd963833075a0dd26e33401d3ba01f57e7ce5 |
| crates/jarde-java/src/build.rs | f4660755cf2ebc699dccf8ee7f7c984a94f6695a22a62cf649d4b728c76a1ade |
| crates/jarde-java/tests/p3_loop_exit_gateways.rs | 519a2ebb0b7a1c2855142b76dfe86c91bc967328dc42f21261c298a641104697 |
| crates/jarde-java/tests/p3_loop_terminal_return.rs | 88b5ad14acbda62a1b4911860e305d63e9c11f8e1d16fcc5fb6ab2e084d1cf7e |
| tests/p3_for_latch.rs | 4e55559b2770c6154e06fb645e543414feff0842819660cde0c3e737007b6550 |
| tests/fixtures/p3-loop-terminal/LoopCases.class | 004e99baeb062861f95a4dac13e3a78399a66efd31f2589b0028cfe50432ae33 |
| tests/fixtures/p3-for-latch/v8/ForBoundaries.class | 9d580094884645a9acd495f604a54f7e8b42e00da837ae99ee184003e560fafe |
| openspec/changes/recover-proved-computed-for-initializers/results/controls-baseline-root-v1/direct-return-actual/javac8/classes/ComputedForDirectReturn.class | 8f68e9bf708ef04bbe726646bfd227fcbe51e80fdda4bd93da04f6d53d644735 |
| openspec/changes/recover-proved-computed-for-initializers/results/controls-baseline-root-v1/actual-controls/javac8/original/ComputedForControls.class | 4c253eb463a8e5bf105c2814e080cf31eb31235dca1a8b98ed7c8d2f3e471dff |
| openspec/changes/recover-proved-computed-for-initializers/results/shared-stack-input-root-v1/javac8/original/classes/SharedStoreInput.class | c887d7f567af2c292ea7e22f99e5a1988150d4b7d4f9f6fb7e9226db7e09fa0f |
