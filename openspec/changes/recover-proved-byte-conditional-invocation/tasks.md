## 1. Byte 条件实参证明

- [x] 1.1 增加独立 DT-28 fixture 和 classfile 证据断言：调用 descriptor 为 `B`，条件两臂均能分别证明可无损地按 byte 呈现；使用固定 original/JADX/Jarde 源码与 source-map 输出核对条件值、分支来源和调用 descriptor，不把新 cast 冒充物理 `i2b`。
- [x] 1.2 在现有调用实参恢复中增加仅限 byte 条件节点的分支证明，并在两臂分别产生可编译的 byte 呈现；定向 Rust 测试须验证字节条件调用正文恢复且不改变条件单次求值。
- [x] 1.3 增加 refusal 反例：错误目标 descriptor、越界常量、无 `i2b`/byte 来源的 int 变量臂、歧义控制流及预算/取消停止均保留完整 fallback；运行对应定向测试并确认 producer/BCI 未丢失。

## 2. 完整源码回归

- [x] 2.1 以原 Java 8 fixture 与固定 JADX 完整源码重编作基准，重编 Jarde 全部相关源文件并以 `java -Xverify:all` 运行；要求输出一致为 `1:0`，且现有直接 primitive cast、移位和 mixed int/long 条件控制仍通过。
- [x] 2.2 运行相关 crate 定向测试、`cargo check --workspace --locked`、`cargo fmt --check` 与 `openspec validate recover-proved-byte-conditional-invocation --strict`；尝试全量测试并把既有独立门禁失败单独记录，保存三方重编、验证运行与来源证据结果。
