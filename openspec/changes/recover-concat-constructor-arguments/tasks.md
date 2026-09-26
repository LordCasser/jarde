## 1. 证据与边界

- [x] 1.1 核对冻结的 `Probe` Java 8 class、三方源码/报告、原/JADX runner 的五行 `arm-2`、Jarde 的两处缺返回及哈希；以 `analysis.md` 复现命令和 `SHA256SUMS.txt` 验证。
- [x] 1.2 固定无拼接的常量/参数构造及独立拼接返回三个对照；用 Java 8 重编与 `java -Xverify:all` 证明这些方法仍可运行。

## 2. 证明计划组合

- [ ] 2.1 将完整的已证拼接计划只读传给对象构造证明，保持链头的独立 `new` 排除；以相邻已有 `concat@1`、`new@1` 测试及新增组合正例验证没有重复认领。
- [ ] 2.2 在外层构造范围内仅接纳实际构造实参依赖中的完整拼接链，核对链尾身份、唯一消费、指令范围/顺序与异常覆盖；用 `thrown`、`constructed` 及错参数、链外读者、多消费、跨范围反例验证接受与拒绝。
- [ ] 2.3 保留对无关分配、独立 void 调用与不能解码/证明指令的拒绝；以定向反例验证原物理 BCI、拒绝原因与消费者来源完整。

## 3. 完整类与主代理验收

- [ ] 3.1 用重建 CLI 不编辑生成文本地输出完整 `Probe`，以 `javac --release 8` 和 `java -Xverify:all` 逐行比对原/JADX/Jarde 的五条输出；核对默认/all 正文与真实来源一致。
- [ ] 3.2 Root 独立审查唯一所有权、异常边界及失败回退，运行相关 Rust/Java 回归、`cargo fmt --all -- --check` 与 `openspec validate recover-concat-constructor-arguments --strict`；记录并清理隔离 Cargo target。
