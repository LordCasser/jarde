# root 独立验收：已证明的 boolean 数组初始化器

本案只扩充既有 `ArrayInitializers` 证明记录：证明循环将每个 `ValueId` 与其真实 store BCI 配对；元素表达式仍在整体消费位置求值，类型转换才使用配对 store。仅在组件已证明为 `boolean`、对应指令确为 `bastore`、解码操作为 int 形状数组写入、值呈现为 B/C/S/I 时，复用已有 `integer_low_bit_boolean` AST。已有 boolean 证明及 0/1 保持直接拼写。闭包、单次使用、顺序、异常处理器、预算和普通 `array_write` 路径都沿用原规则；没有添加 pass、IR、类型推断器或降级为先赋临时变量。

## 冻结输入与三方结果

root 从永久 Java 8 源码独立重编并运行受控补丁脚本，得到的两个 class 与永久 fixture 逐字节一致。`BoolInit` 原/补丁 SHA-256 分别为 `5304fa21e9594a0ab6e04e0f79ee0da3bbcdcbc85e3019183d7cdd5e7b22da7d` / `e0e8cf5cf99fdb3e402db16e6d0b86db1b8b8672df2e684523c72a67df724a3f`，189 字节仅在偏移 93、161、165、169、173、177、181 改变：返回 descriptor、`newarray` atype 与五个 `iastore→bastore`。`BoolInitEffectful` 原/补丁 SHA-256 为 `b7e8c4936adab5af0e18f7df86480e1ef72ff79e22e935d8a6e66a99177214bd` / `ea2175192c0967da85ef5d78253bd458fb8a5543bcaf21f622b20a2caf59a8d0`，614 字节仅在 334、478、486、494、502 改变。两份补丁 class 都通过 `java -Xverify:all`。

root 以最终私有构建 CLI 对两份永久 class 执行 `class-source --policy single-class --evidence all`。完整 Jarde 源码 SHA-256 分别为 `328623466548e1f82a9c6755ea17226d9fff9c662e7e24d7753cdad5a0c87b86` / `06fa6954c720c0cbdf1dc6f3e6fc53efa070b706e05c91c020171805fface9d5`，零 `@bytecode` 和 `not recovered` marker；分别以 `javac --release 8 -g:none` 编译完整类和 runner，再以 `java -Xverify:all` 执行，输出与原补丁 class 逐字节相同。前者五元素为 `[false, true, false, true, true]`，输出 SHA-256 `5eb544d91affd86d74eb4a839f6c966e822e83055515f9a01b88687bdfcdc6a6`；后者五行输出 SHA-256 `9af9e18526f357ba9f2288520cd69520b2a7085de9fb7a5f2264d7d4f9f129e7`，正常路径 trace `abc`，三个除零路径分别 trace `a`、`ab`、`abc`。这是可观察调用顺序与异常前副作用相同的范围内证据，并未宣称任意跨块初始化器可折叠。

root 同时以 JADX 1.5.6 原样反编译这两份 class：输出分别保留 `new boolean[]{0, 1, 2, 3, -1}` 与 `new boolean[]{element(i, 0), element(i, 1), element(i, 2)}`；`javac --release 8` 各报 5/3 处 int 不能转换为 boolean，均退出 1。未修补 JADX 源后再执行。修前 Jarde 的 consumer BCI 23 拒绝及审计时的 JADX/JVM 阶段保存在[独立审计](../../evidence/java-syntax-2026-09-25/boolean-array-initializer/analysis.md)，不以修后的 CLI 充当旧基线。

## 来源、边界与门禁

root 审读了证明循环中的 `(stored_value, store.bci())` 顺序配对和 `array_initializer_element` 的准入。合同测试对 2、3、-1 的各自转换片段断言来源仅含对应 store 14/18/22、不含整体 consumer 23；effectful 三片段各含自己的 producer/store 10/18/26，且正文中各调用一次、顺序一致。0/1 的直接 boolean 拼写不强造单独转换节点，其 store 保留于整体 `NewArray` 来源；测试没有把聚合 `of_bci(store)` 误当成逐元素配对证明。单条错 opcode 使整个 initializer 证明失败；default/all/replay 同正文，紧预算/预取消没有部分正文或来源表。

新 `jarde-java` 合同 5/5、完整类 JDK 集成 2/2；10 组相邻普通测试均通过，另显式运行窄数组、boolean 字段、窄返回三项 JDK ignored 测试均通过。`p5_corpus_fingerprint` 为 556 个文件重生成并验证 5/5。`cargo fmt --all -- --check`、`git diff --check` 和 `openspec validate recover-boolean-array-initializers --strict` 通过。严格定向 Clippy 先遇到 enum/region/report/build/reuse 的 13 条既存警告；只对既存的五类 lint 临时豁免后，本案库合同与完整类测试目标均通过，没有新增告警。普通数组写入和 byte[] 的规则保持独立；未知组件、跨块/跨异常处理器、非 B/C/S/I 呈现值及未通过闭包证明的链继续拒绝，不从 opcode 单独推断数组类型。

验证结束后清理了实现代理的私有 Cargo target（`cargo clean` 移除约 4.1 GiB）和 fixture 代理的私有构建/复放目录（约 1.8 GiB；该目录兼放非 Cargo 复放文件，没有 Cargo 的 `CACHEDIR.TAG`，在核对路径后删除）。仓库默认 `target/` 此轮未重建；项目约 220 MiB，本机可用约 99 GiB。
