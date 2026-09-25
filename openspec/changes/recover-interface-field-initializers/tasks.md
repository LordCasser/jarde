## 1. 冻结合法输入与投影边界

- [x] 1.1 `interface-field-initializers/` 保存 Java 8 原类、JADX/Jarde 完整源码、`javap`、编译/验证执行与字段表交换补丁；root 在独立目录原样重放，`summary.json` 逐字节一致。原/JADX 两份均编译并输出 `ABT|A1|B2|4|7`，Jarde 两份均因三处缺 `=` 与接口 `static {}` 编译失败，Code 未被补丁改动。
- [x] 1.2 `boundaries/` 冻结额外独立副作用、重复/漏写字段和分支三种有效 Java 8 class，保存原 class/JADX/Jarde 整类源码、编译状态与真实 BCI；root 修正 JADX 同包 support 后在独立目录原样重放，三份 `-Xverify:all` 均成功，`summary.json` 字节相同，两个补丁原运行分别为 `ABXB|A|B` 与 `AB|null|B`，分支为 `CAB|A|B`。分支只证明当前直线准入不覆盖它，不宣称所有分支无法表达。
- [x] 1.3 将最小正例及关键拒绝边界加入永久 Java 8 fixture，并以 reader census/fingerprint 与原始运行输出确认输入冻结，避免把实现中的目标代码执行混入产品能力。`p3-interface-field-initializers/` 共 9 个 Java 8 class、24 个 Code，root 独立重编正例并重放两项补丁/验证执行；reader census `(151, 1086, 98, 346, 8)` 与 corpus 368 个文件的 5 项指纹检查通过。
- [x] 1.4 `forward-binding-exception-edge/` 冻结限定名前向读取与真实异常表边界；root 独立复制重放两类 `summary.json` 逐字节相同。字段表换序后原 class 验证执行 `L|0|9`，JADX 完整源码编译验证却错成 `L|9|9`，Jarde 编译失败；带一项异常表的受控 interface class 验证执行 `ABEC|A|B`，JADX/Jarde 整类编译失败。前者说明必须证明源码重排对默认值读取的影响，后者保守拒绝而非宣称所有 handler 不可表示。
- [x] 1.5 `constant-phase-boundary/` 冻结 `<clinit>` 常数写入却无 `ConstantValue` 的合法 Java 8 interface：496 B class 通过 JVM 验证且运行 `0|9`，JADX 完整源码编译运行却得 `9|9`，Jarde 编译失败；手写机制对照证明 javac 会生成 `ConstantValue` 并内联。root 使用独立 `--out` 重放，summary、冻结 class 及两份运行输出逐字节相同；此样本只约束投影，不把手写源码冒充 Jarde 恢复。

## 2. 同次成员恢复中的有界类级投影

- [x] 2.1 在现有 `<clinit>` 恢复接缝取得结构化、按预算计费的候选归属，不从 `RecoveryReport.text` 反解析、不重读 class 或重复恢复；用身份/来源定向 Rust 测试核对 owner/name/descriptor、BCI 和真实停止。
- [x] 2.2 对普通接口完整字段集合与 `<clinit>` 做全组证明：自身字段唯一写入、`ConstantValue` 分离、直线路径、无额外效果、值类型及一次求值；原本无 `ConstantValue` 的运行时 RHS 还须由现有 AST/字段事实证明源码不会成为 Java 常量表达式，否则整组拒绝。正例、1.2 负例与 1.5 阶段反例的 Rust 测试证明不作部分投影。
- [x] 2.3 将已证 RHS 用现有表达式发射器写入对应字段声明，文本按真实写入顺序而 JSON 字段保留物理表序；重排字段表样本的完整 Java 8 重编译、`-Xverify:all` 和 trace 与原 class 一致，来源与 `<clinit>` 原报告仍可对照。`cargo test -p jarde --test interface_initializer_projection --test interface_initializer_proof` 两个定向目标均通过（投影 2/2、整组证明 4/4）；投影测试对原类与字段表换序类做 `javac --release 8` 全类重编译及 `java -Xverify:all`，均得 `ABT|A1|B2|4|7`，另验证限定名前向读重编译后仍为 `L|0|9`。root 独立从 CLI stdout 重编主正例并验证执行，结果相同。
- [x] 2.4 核对默认/all 证据、输出/IR 预算、取消及失败路径的原子性；定向测试显示任何中断均无半组初始化式，拒绝/stop 与原始效果保留。默认与 all 证据得到相同完整投影；输出字节与 IR 项预算在投影/证明阶段停止时，运行时字段均没有半组初值，原 `<clinit>` 报告及静态块效果仍在；预取消不发布类源码报告；结构性拒绝保留原 `<clinit>` 文本。`cargo test -p jarde --test interface_initializer_projection --test interface_initializer_proof` 通过（6/6、4/4）。

## 3. 三方对照与独立验收

- [x] 3.1 用修后冻结 CLI 原样重放 1.1–1.5，逐一保存原/JADX/Jarde 的完整类编译及原类/修后类的验证执行；证据在 `evidence/task-3-1-replay-final/`，CLI SHA-256 `b2aaa990e4811fdbec5bbf5e7781c97ad4da63dbe988e5865f46fca6e008902c`。正常和字段表换序原/JADX/Jarde 均编译、验证执行 `ABT|A1|B2|4|7`，无接口 `static {}`；限定名前向读原/Jarde 为 `L|0|9`，JADX 错成 `L|9|9`；四个拒绝边界未误称整类恢复，常数阶段原为 `0|9`，JADX 错成 `9|9`，Jarde 保守拒绝。
- [ ] 3.2 root 独立审查跨成员身份、字段顺序、常量早期初始化、表达式效果位置与来源，并在复制目录重建 CLI、重放正负证据和相邻普通类/枚举/注解类回归。
- [ ] 3.3 root 跑相关 Rust/Java、reader census/fingerprint、`cargo fmt --all -- --check`、严格 Clippy 与 `openspec validate recover-interface-field-initializers --strict`；只将本项失败修入本 change，独立架构债务另记。
