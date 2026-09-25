# Root 独立验收记录

## 返回位置与算法

2026-09-25，root 审读了 `build.rs` 的四种返回消费入口。普通与同步返回在实际 `ireturn` BCI 渲染；switch 栈汇合在各臂的求值 BCI 渲染值，再以共同的真实 `ireturn` BCI 判断窄化；字段前/后自增先保留一次字段更新表达式，最后才适配返回类型。新适配只在方法 descriptor 为 B/C/S、实际指令为 `0xac`、值已呈现为 B/C/S/I 时用现有 Cast 陈述 JVM 返回窄化。它不放宽普通赋值、字段写入或调用位置，也不更改值生产者的类型。缺少整数呈现类型与 boolean 操作数仍拒绝。

本地 JADX `InsnGen.java` 的 RETURN 分支直接打印已有参数表达式；在下列输入中没有补上必要的 B/C/S 窄化。Jarde 将转换放在真实返回消费处，既保留上游的静态类型，也覆盖 switch、同步与字段更新的特殊入口。

## 完整类执行对照

root 使用私有 CLI `/tmp/jarde-narrow-return-impl-target/debug/jarde-cli`，SHA-256 `f9adc2ccd7c89a8e87225861badbadd67c56ed9b1b5cc65c13ba014ac37a9730`，在隔离临时目录重新编译源码、精确补丁 descriptor、执行原 JVM、生成 Jarde/JADX 完整源码并用 `javac --release 8` 与 `java -Xverify:all` 检验。旧 evidence 脚本的输出目录及 CLI 路径仅在临时副本改写，没有覆盖历史证据或使用共享 `target/`。

| 输入 | 原 JVM/Jarde 逐行相同 | Jarde 完整类 | JADX 完整类 |
| --- | ---: | --- | --- |
| `NarrowIntegerReturns` 永久 1288 B 主类 | 49/49，输出 SHA-256 `113187caf92427ac3fdb41c314d958cada2d708d4d69bc2ca39cbabce7abd967` | 0 引用，Java 8 编译及验证执行通过；源码 SHA-256 `038866fd6ef87782f36da67f14134da1f72030b6c2c5c21887fa88ef05c31620` | 既有三方基线完整编译失败 |
| `narrow-locals`，SHA-256 `833415dd5400ba5e477e659379b0bdadad8df0bc5c903cb7a9706467a9946368` | 20/20，`122de01f39e3e0e850496f1d3c91f03c56aaf33c36dac0f86ae3190098fe6e34` | 0 引用，编译/执行通过 | 编译/执行通过，20/20 |
| `return-sinks-core`，SHA-256 `dbb7b8cecab3df4d6fcb207cf4db7bbe9337af73d8ceeba8df9f9926087002d5` | 19/19，`7f51301ec49f5c5a87b979129e3a56a44fe58fc620ee66d8a59386947ccf6ba2` | 0 引用，编译/执行通过，字段结果与同步 null 异常相同 | 编译失败，未运行 |
| `return-narrowing` 的 B/C/S 三份 descriptor 变体 | 13/13 各一份，共 39/39 | 每份 0 引用、编译/执行通过 | 每份编译失败，未运行 |
| `actual-stack-join` 的 B/C/S 单 descriptor 变体 | 12/12 各一份，共 36/36 | 每份 0 引用、编译/执行通过 | 每份编译/执行通过 |

`actual-stack-join` 的源码由 Java 8 编译，受控补丁生成 verifier-valid、major 49、无 StackMapTable 的真实 operand-stack 汇合；三个变体 SHA-256 分别为 `ad9b88a41ec56ed7a5ff553388dcf3867f8d00bc5a6a27404da4976b7b684149`、`c5c2ab16469319970488a56cb881cb14873a23c55e59e77b4dd582f1dd3b7abe`、`a5f141e48dc0ee777e02d9cd4c87c25a27ca3b9b40f86909f8f05e2153196dd3`。它们是 Java 8 兼容 class，并经当前 JDK 严格验证；不能称作 major 52 原 class，这一降版本仅是样本对 StackMapTable 的受控处理。B/C/S 运行输出 SHA-256 分别为 `379b9657728752f92180a8a08b86d14190f1e8fb3e984f4641ae78801c179b3e`、`291fb04043315429b2dd1b4217456d6bd6b480a9f7fa116cf80d96b80713c6c7`、`d4fac0a353e0cdeaf8c00133c182d6b3a55f8b64e16c4eafa9e158560a2d585b`。

`return-narrowing` 的 `(I)Z` 变体保持拒绝；13 项原 JVM 边界未算进 B/C/S 正例。root 另以 Java 8 class 的唯一 `(I)I` descriptor 改成 `(I)Z`，在 `java -Xverify:all` 下确认输入 2/3 分别返回 false/true，符合 [JVMS `ireturn`](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.ireturn) 的 `value & 1` 规则。该可恢复场景单列后续变更，不能靠把任意非零值写成 true。

## JADX 的 boolean 到 B/C/S 错误边界

本地 JADX 1.5.6 对合法 `(Z)B/C/S` 方法的 boolean 参数 `ireturn` 分别写成 `return z ? (byte) 1 : (byte) 0;` 等三元式。root 以 Java 8 编译源码、只改目标方法 descriptor，并将调用者唯一 `iconst_1` 改为 `iconst_2`：原 class 和调用者均通过 `java -Xverify:all`，B/C/S 三种返回均为原始值 2；对应 JADX 完整源码也通过验证执行，但三种都返回 1。两组调用者和目标类都没有改 Code 长度。JVM 的 B/C/S `ireturn` 按 `i2b/i2c/i2s` 处理整数值；boolean-presented 参数的 Java 三元表达式会把非规范 int 载荷归一化。Jarde 当前保留可定位的拒绝，比输出错误行为更合乎本项目的语义契约。此边界不得混入已恢复的整数值 B/C/S 正例。

## 来源、停止与相邻回归

root 复跑 `p3_narrow_integer_return_contract` 3/3：默认/all/replay 正文一致；直接、同步和字段自增 Cast 的 source-map 同时包含操作数来源与真实返回 BCI；紧输出预算和预取消均不发布半个转换或半张来源表。永久主类 `p3_narrow_integer_returns` 2/2，另 ignored Java 完整执行 1/1 通过；`p3_deferred_value_order` 的 ignored 完整执行 1/1 通过。

root 另独立复跑 `p3_narrow_return_boundaries` 3/3：字段自增及同步返回的值来源、写入/退出和真实 `ireturn` 均有 source map；共享 switch 的 B/C/S `ireturn` 来源与原 JVM 输出一致；boolean→B/C/S 保持可定位拒绝，原 JVM 的 raw 2 输入仍输出 2，整数→Z 则按最低位输出 false/true。使用 `regenerate_task12_fixtures.py` 从 Java 8 源码与受控补丁重建 5 个永久 class，逐字节等于仓库样本；三个 stack-join hash 与上文相同，boolean 目标与调用者 SHA-256 分别为 `336f8f291987fa9fdc583e62ffb99b522d37d19ffef8db6cfd64e5d77db973a4` 和 `e791af87e373254d2c107956813488042258894553f200fd5cfe0957805cbcc1`。

相邻非 ignored 测试通过：`p3_boolean_contexts` 13/13、`p3_field_increment` 2/2、`p3_guard` 13/13、`p3_sync_return` 3/3、`p3_switch_value` 2/2、`p3_deferred_value_order` 2/2、`p3_meeting` 5/5、`p3_required_conversions` 7/7；最后两组各跳过一个已有失效断言：`p3_meeting` 仍要求已恢复的 `i2l` 为拒绝，`p3_required_conversions` 的 `ir_items` 冻结计数仍写 224，而共享 build 现为 234。两项债务已在 roadmap 单列，不在本案修改语义或数字以掩盖。严格 OpenSpec 校验与 `git diff --check` 通过。

`cargo fmt --all -- --check` 通过。严格 `cargo clippy --workspace --all-targets -- -D warnings` 先碰到 13 个本案以外的既存 lint；修复本案新增的 `let_and_return` 后，仅对本案三个测试目标重跑 Clippy，并临时豁免五类既存 lint，通过。全目标 Clippy 随后暴露其他未闭合变更的测试编译错误（`recover_for_class_source` 参数数目、`spell_ordinary_signature_type` 参数数目）和一个相邻测试的 `cloned_ref_to_slice_refs`；这些不在本案扩大修改。

root 统一重建 `tests/fixtures/corpus-fingerprint.json`，随后 `p5_corpus_fingerprint` 5/5 通过；三组本案测试合计 8/8、另 ignored 原/Jarde 完整类执行 1/1 通过，最终 `cargo fmt --all -- --check`、`git diff --check`、OpenSpec strict 均通过。`/tmp/jarde-narrow-return-impl-target` 与代理 `/tmp/jarde-task12-target` 已分别通过 `cargo clean --target-dir` 清理；项目目录约 464 MiB，磁盘剩余约 100 GiB。本 change 的 8 项任务据此完成；全目标 Clippy 与两条既存相邻断言债务仍按 roadmap 分案处理。
