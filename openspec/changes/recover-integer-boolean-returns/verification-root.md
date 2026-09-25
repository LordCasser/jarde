# root 独立验收：`ireturn Z` 的整数最低位

本变更复用 `return_expr` / `adapt_return` 的真实返回消费位置，以及 Z 字段写入已有的 `value % 2 != 0` AST 构造。准入仅在方法描述符返回 `Z`、实际指令为 `ireturn` (`0xac`)、被呈现值为 B/C/S/I 时成立。原值只在一棵子树中出现一次；返回 BCI 作为派生来源保留。普通、同步、switch 合流及字段自增返回走同一转换；已证明 boolean 的快路仍保留。未扩展赋值、参数调用、`bastore` 或局部类型推理。root 审读了 `build.rs` 的返回入口、字段自增快捷入口、`field_value` 及来源 helper，并以新契约测试复核。

## 冻结产物与三方完整阶段

root 使用独立临时目录执行 `regenerate.py OUTPUT_DIR`，与永久文件逐字节比较，两个 class、两个补丁 JSON、两个 `expected.txt` 共 6 项全相同。主类 SHA-256 为 `5b993e7c5b28588ded97dc53b6982608895745fdd625a77e0feb69ee6783eade`，补丁前后全部方法 Code SHA-256 均为 `347f163f2497b8b8cb0a3cd7f246f0ee9db9179a4c7660d68df506301bdff854`；major-49 共享返回变体 SHA-256 为 `3b727e5b7bb8b630eca95ada5f0ff6516063e58fa94718ba36cf1c649ca107e2`。原 class 和 runner 经 `javac --release 8 -g:none`、`java -Xverify:all` 成功。

root 用私有构建的 `jarde-cli class-source --policy single-class --class IntegerBooleanReturns --format text --evidence all` 独立生成完整类；输出 SHA-256 为 `4c061ac70c21736e53754a4dc81003119775b91e15541ed44341695e3d2d3da5`，`@bytecode` 次数 0，六个目标方法各有一次 `% 2 != 0`。此完整源码以 `javac --release 8 -g:none` 重编成功，并用同一 runner 在 `-Xverify:all` 下运行。原 class、Jarde 重编 class 和永久 expected 的 49 行逐字相同，输出 SHA-256 `7f8ecffdf4d167c21382ac0bf3fa1b2dc4215b9bebb1b35570f07f0d53b89e5e`；其中包括 0、1、2、3、-1、-2、整数极值、单次调用计数、字段前/后自增后的完整整数值及 `syncOn(null, 1)` 的异常类。

安装的 JADX 1.5.6 对同一 class 成功生成源码，原样 SHA-256 为 `e18dbd58a291b6e2d9bd73a70a7b2ec5cd11bac893e11a3e1c156326890e5c71`，与冻结文件逐字相同。`javac --release 8` 对该源码失败：`post` / `pre` 有 `?? r1`，而普通方法还直接写 `return i;`。没有修补其输出再执行。旧 `(Z)B/C/S` raw-2 调用者继续保持 Jarde 可定位拒绝；该相反方向的非规范 boolean 载荷不能按本项整数返回的规则归一化。

## 来源、停止及相邻行为

新契约测试 5/5 检查六个方法的原操作数与真实 `ireturn` BCI、switch 两臂与共享 BCI、默认/all/replay 同正文、raw-2 负例，以及输出预算/预取消不发布半成品。新完整类和 major-49 边界测试分别 2/2、1/1；旧 `p3_narrow_return_boundaries` 3/3。旧 `p3_boolean_contexts` 中原先把整数 `ireturn Z` 一概视为拒绝的两条断言，已据本次原 JVM 最低位证据改为检查明确转换；其余 boolean 赋值/调用拒绝仍通过。

root 复跑 14 组非 ignored 相关测试，覆盖 boolean field/context、字段自增、同步、switch、窄整数/数组、延期求值与调用参数，全部通过。另显式运行五组需要 JDK 的 ignored 原/Jarde 完整执行测试，全部通过。`p5_corpus_fingerprint` 已按本次受控 fixture 更新并通过 5/5。

严格 Clippy 在 `jarde-java` 遇到 14 处警告，其中本项 `return_expr` 的一处 `collapsible_if` 已修；其余是既存的 `useless_conversion`、`too_many_arguments`、`type_complexity`、`needless_option_as_deref` 和 `collapsible_if`，不混入本项架构范围。只对本项三个测试目标豁免这五类既存警告重跑 Clippy，通过。全目标 Clippy 另有既存测试编译缺口，见相邻变更记录；本项未扩大处理。

`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-integer-boolean-returns --strict` 均通过。root 统一重建 `tests/fixtures/corpus-fingerprint.json` 并更新 fixture 索引。停止构建后使用 `cargo clean --target-dir` 清理 root/impl/fixture 三个私有目录，分别移除 3.1/1.5/1.3 GiB；项目目录约 465 MiB，磁盘可用约 99 GiB。本 change 的八项任务已完成，下一项 `boolean[] bastore` 独立分析、独立成案。
