## 1. 冻结完整类与桥接边界

- [x] 1.1 保存普通接口覆写、附带副作用桥接、无继承需求的纯转发桥接三份 Java 8 class 与 source-only runner，核对 class SHA、Code、真实 flags 与 `-Xverify:all`；用固定 CLI/JADX 原样生成完整类，记录 javac/JVM 结果。root 在独立目录和证据目录重放 `replay.py`，三组 `summary.json` 字节相同；手删桥接声明的方案实验单独标记，不算产品输出。
- [x] 1.2 将三组 subject class、正例接口和 source-only runner 纳入永久 fixture；root 在独立临时目录重编/补丁后逐份核对四个 class SHA、`-Xverify:all` 的 `value|value|value`、`value|value|1`、`value|value`，并用 `javap` 核对 bridge flags、纯转发与附带效果。reader census 158 class / 1,106 Code 通过；corpus fingerprint 重生后 388 文件、5/5 检查通过，10 个桥接 source/class 文件入清单；A01–A18 验收行保持原样，不把 JADX 改名后的可编译性当作语义正确。

## 2. 同次桥接结论到类级证明

- [x] 2.1 在既有 `bridge@1` 与 class-source 同次恢复接缝提供结构化目标身份、调用 BCI 和纯转发 verdict，尤其补足正例无 `checkcast` 的 `Ok(None)` 路径，不解析正文、不二次恢复；定向 Rust 测试核对三类 class 的原方法身份、flags、目标及拒绝来源。root 重跑 `cargo test -p jarde-java bridge --locked`：unit 2/2、pattern 8/8，通过同次 sidecar 与 essential/all 独立性测试；普通恢复仍不收集 class-source 候选。
- [x] 2.2 对当前类唯一源级目标、bridge name/参数/返回关系、无独立效果/handler/不可重建元数据，以及已解析父类/接口擦除需求做整项准入；正例准入，`negative/` 和 `orphan/` 各自按正确原因拒绝，缺失/歧义依赖拒绝，定向测试证明任何一条条件都不能单独许可。root 审读 class-source 交叉证明并重跑 `cargo test -p jarde --test class_source bridge --locked`，3/3 通过。
- [x] 2.3 仅将准入的 bridge 从完整 Java 方法声明投影为有身份的说明，`ClassSourceReport::methods` 保持物理表序及原 `RecoveryReport`；完整正例重编后用 `javap` 核对新 `ACC_BRIDGE get()Object`，直接/接口/擦除调用 `-Xverify:all` 输出与原类一致，负例不被隐藏或改名。定向测试核对物理报告、JSON 和新 bridge flags；root 用 CLI `a3a29b59…` 原样重放完整正例，零引用、javac exit 0、JVM `value|value|value`。
- [ ] 2.4 为候选、继承读取、投影记录和输出/来源逐项计费并轮询取消；essential/all 正文一致，低工作/IR/输出预算或取消时无半投影，物理成员与停止原因仍可查，定向测试覆盖。

## 3. 三方重放与主代理验收

- [x] 3.1 用修后冻结 CLI 原样重放 `1.1` 三组完整类并保存输出；正例零引用、javac 成功且原/JADX/Jarde 值和擦除入口一致，两个负例不宣称等价投影，不能手改 Jarde 源码作为实现验收。`bridge-source-projection/replay-accepted/summary.json` 固定 CLI SHA `a3a29b59…`；正例原/JADX/Jarde 均重编并输出 `value|value|value`，负例 Jarde 保留未投影声明而不能重编，JADX 负例亦未证明语义等价。
- [ ] 3.2 root 独立审查桥接目标、继承需求、类型/效果/元数据边界与物理报告，复跑相邻 bridge/call/accessor/类源码/预算测试；同一 JAR 与未知依赖环境均验，未证明的形状继续拒绝。
- [ ] 3.3 root 运行 Java 8 永久 fixture、reader census/fingerprint、`cargo fmt --all -- --check`、严格 Clippy 和 `openspec validate project-proved-bridge-forwards --strict`；本项失败只修本项，独立债务另列并在验证记录写明。
