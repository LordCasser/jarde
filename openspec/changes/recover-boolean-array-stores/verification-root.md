# root 独立验收：已证明 `[Z` 的真实 `bastore`

实现只在 `array_write` 的 `Some(Type::Boolean)` 分支增加一个准入：已有 boolean 证明直接写；否则要求当前真实 opcode 为 `0x54` 且值完整呈现为 B/C/S/I，使用已验收的 `integer_low_bit_boolean(value, at)` 构造一棵值子树。现有 `array_element` 与 `array_store_opcode_matches` 已先证明组件和 opcode 一致；没有从 `bastore` 猜 `[Z`、扩展 `meeting_position`、改 `byte[]` 路径或添加 AST/IR/通用 pass。root 审读了数组、下标、值的原渲染顺序、store 来源和低位 AST，三者仍在 `IndexAssign` 的原位置求值。

## 冻结与三方执行

审计原/补丁 `RawBool` SHA-256 分别为 `e38d0e46a055d5183001d2e51ed329c2a0f121409f27ead257d34f6ab76317e3` / `32286d138ace6a328a8c5ba7d0a0d66fadb4bf9dbbab79385e6dc3a01dcd6c2b`，只差同长 descriptor 的两个字节与 `iastore→bastore`、`iaload→baload` 两 opcode。`Order` 原/补丁 SHA-256 分别为 `63dc18e93763ffbf9d6ce73a299a7d140023eb278dd2d430f6e47d208bb55ef0` / `31d8fc3394c39cfe9a9e62da435af9245d2f26cd24e62e077e7310e1bf97d414`，只差同长 descriptor 的四个字节与两 opcode。root 独立核对永久 class 与审计 hash、实际字节差异，并复放 `java -Xverify:all`：`RawBool` 8 行输出 SHA-256 `c92153475136dcf4b0a9a8774debf3291b786db7badb26fd91098d1d83ad5a1d`；`Order` 4 行输出 SHA-256 `64236cae8680726122634fdc66fd5955b93e9ccb578d83bda6f4221d299d4e84`。

root 私有构建 `jarde-cli` 后，对两份永久 class 各生成一次 `class-source --policy single-class --format text --evidence all`。`RawBool` 源码 SHA-256 `65f094858f4c04cf78ea2f78c6888cc6955fca90cceb549bb99113e444032da3`，`Order` 源码 SHA-256 `a5c922ebeee0bc890d92a09660e0c7ce4dbbfa08f62b7a2c0da20515291284ad`；两者各含一次 `% 2 != 0`、零 `@bytecode`。分别用 `javac --release 8 -g:none` 编译完整 Jarde 类和原 runner，再以 `java -Xverify:all` 执行，分别逐字匹配原 class 的 8 / 4 行。Order 成功、null、越界、值 producer 异常均为 `trace=123`，异常类型和返回/状态一致，证明没有提前或重复求值。

JADX 1.5.6 原样源码 SHA-256 分别为 `a2c78853f4f9e485f76ebce01559ad428848ad958e3136bd9791edcb598e7790`、`ba64d98c8d2a93d22576b6d5b05e7c4f600fff4a447ceb359f9ab9b94edfc091`，与冻结审计完全相同；两份 `javac --release 8` 均因 int 赋给 boolean 退出 1，没有修补源码后继续测。JADX 的数组组件类型传播可借鉴，直接打印 APUT 赋值在此处不正确。

## 来源、拒绝及相邻行为

新合同测试 6/6 覆盖 B/C/S/I 呈现、已证明 boolean 快路、`RawBool` operand/store BCI、`Order` 的三 producer 与 store BCI、default/all/replay 同正文、预算/预取消原子停止。永久整类测试 1/1；旧 `p3_narrow_array_stores` 中先前要求 `[Z` 整数写入拒绝的断言按上述原 JVM 证据改为检查最低位，旁边 boolean→byte[] 的拒绝保留。root 另独立运行既有 `run_fixture.py --jarde-cli ...`：合法的未知组件 null-array `bastore` 通过 JVM 严格验证并抛 NPE，Jarde 仍在 BCI 5 拒绝；已证明 `[B` 的窄写入和 boolean→byte[] 边界未放宽。字节码 verifier 对 `bastore` 的值要求 int 形状，不能构造合法 long/float/reference 栈值作正例；代码上的呈现类型门槛仍明确拒绝这些类型。

root 复跑 11 组非 ignored 相关测试，涵盖新合同/整类、数组读/初始化器、boolean field/context、窄数组、整数→boolean 返回和延期值，全部通过；另显式运行 boolean field、延期值、窄数组三组 JDK 完整执行测试，全部通过。`p5_corpus_fingerprint` 随本案永久 fixture 更新并通过 5/5。

[独立初始化器审计](../../evidence/java-syntax-2026-09-25/boolean-array-initializer/analysis.md)证实 `new boolean[]{raw int}` 仍在已折叠初始化器的 consumer BCI 23 拒绝，而真实 stores 为 6/10/14/18/22。它不属于本次普通 `array_write`；后续 change 应在已有证明计划中保留元素到真实 store 的配对，不混入本项。严格 Clippy 仍先遇到 13 条已记录的 enum/region/report/build/reuse 既存警告；本案两个测试目标对这五类既存 lint 临时豁免后通过。本案没有新增 Clippy 告警。

`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-boolean-array-stores --strict` 均通过；永久 fixture 索引及 `tests/fixtures/corpus-fingerprint.json` 已更新。构建停止后用 `cargo clean --target-dir` 分别清理 fixture 与 impl 私有 target，移除约 2.6 / 1.2 GiB；初始化器审计代理另清理约 1.0 GiB。项目目录约 465 MiB，本机可用约 98 GiB。本 change 的八项任务完成；全工作区既存 Clippy/旧测试签名债务不在本次扩张。
