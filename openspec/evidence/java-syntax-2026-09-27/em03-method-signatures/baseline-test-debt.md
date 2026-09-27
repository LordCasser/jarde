# 独立基线测试债务：`member_inner` 调用缺参

`origin/main` 基线 `33f15fde5847ff150bf6689fc990866afd08bc13` 上，`cargo test -p jarde --lib --no-run` 在编译测试时失败。`src/member_inner.rs` 测试约第 2600、2611、2627 行分别调用 `capture_write_is_unique_site` 或 `scan_capture_method_uses`，遗漏现有函数签名要求的 `write_bci: u32`。完整编译输出见 [baseline-lib-test-error.log](baseline-lib-test-error.log)。

该错误与 EM-03 的成员参数属性读取及源码命名无关，需单独修复并重跑 `jarde` lib 单元测试。本变更没有修改 `member_inner.rs`；EM-03 新增于 `src/class_source.rs` 的两个单元测试因这个基线编译错误尚未执行，等该债务闭合后应一并运行。相关行为另由本次 `tests/class_source.rs` 集成回归覆盖并已通过。

后续独立测试提交 `0a4ac99e` 为三处调用补入已证写入 BCI `2`，未改变生产函数；root 的 `cargo test -p jarde --lib` 随后 135/135 通过，包含 EM-03 新增的两个单元测试。此段保留原始阻断事实和分拆处理记录。
