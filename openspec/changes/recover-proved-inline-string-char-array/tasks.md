## 1. 共享现有证明

- [ ] 1.1 核对 `ArrayInitializers`、`init::sites` 与 builder 的证书、预算和停止传递路径，记录准确的单次证明交接点；以 EM-27 固定 `javap` 和既有先存局部形态确认顺序。
- [ ] 1.2 将既有数组初始化证明提前到构造站点证明之前，给 `init::sites` 提供最窄只读证书查询并把同一计划交给 builder；用定向测试确认只证明和扣费一次、原有数组初始化与先存局部构造不回退。

## 2. 内嵌构造闭环

- [ ] 2.1 只对准确 `String.<init>([C)V` 的 Java 8 单块直接返回形态，验证唯一 `char[]` 实参、SSA/消费者、父子 BCI 顺序、效果与 handler 覆盖后允许外层构造消费已证数组；定向测试须输出显式 `new String(new char[]{...})` 并保留原有来源锚点。
- [ ] 2.2 加入额外数组读取或写入、错 owner/描述符、第二消费者、不完整顺序或异常范围的拒绝测试；各形态不得发表半个 `NewArray`/`New` 表达式，物理回退及拒绝原因可见。
- [ ] 2.3 验证数组、构造和输出阶段的低预算/取消，以及普通 builder 拼接、非 String 构造和已有数组单元回归；停止状态与物理 BCI 必须保留，`cargo test -p jarde-java` 通过。

## 3. 固定对照与收束

- [ ] 3.1 使用冻结 JADX 和 EM-27 `replay.py` 重放原/JADX/Jarde 完整 Java 8 源码重编及 `java -Xverify:all`；Jarde 全类须编译并与原 class 的八行运行输出相同，两个 `new String(char[]) == "abc"` 都为 `false`，记录固定 JADX 的 `true` 身份反例。
- [ ] 3.2 运行 `cargo fmt --check`、`cargo check --workspace`、相关 `class_source` 回归和 `openspec validate recover-proved-inline-string-char-array --strict`，将命令、版本、哈希、结果写入 EM-27 验收证据后才勾选任务。
