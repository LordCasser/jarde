# Integration 测试实现记录 v1

本记录对应本片 integration 测试编辑。只修改了 `tests/p3_heterogeneous_array_initializers.rs` 与本文件；没有改产品、fixture、OpenSpec 计划/规范，也没有运行 Cargo、Git、rustfmt、Java 或 JADX。所有新断言仍待 root 串行验收，不能据此称实现或任务已通过。

## 已编辑的验收入口

- `direct_new_family_pins_recovered_constructor_methods_and_remaining_controls` 不再把 `numberGridDirect`、`collectionGridDirect`、`ownGridDirect` 作为旧 fallback。现在三者要求 `Structured`/`Java`、没有 `@bytecode`/`jarde_refused_body`，并检查其数组构造、元素生产、实际 `aastore` 和 `ownGridDirect` 的 object allocation/dup/constructor/argument source-map BCI。
- 对 Number grid 检查 Long 子项保留 `(long) mark(2)`，并检查 i2l 的物理 BCI 33；没有把原字节码转换改写为 `2L` literal。每个 grid 断言 `mark` 恰出现两次。
- `ownGridDirect` 的两个 `DerivedA`/`DerivedB` constructor records 从原拒绝清单迁为已呈现 records，并锁定 new、dup、参数、constructor、child store 以及两层 parent/child 数组 store 的 BCI。Collection grid 固定 `Collection[][]`、`ArrayList[]`、`HashSet[]` 和真实生产调用文本；没有放宽 Signature 规则。
- 正例入口补齐 direct 每条 leg 的完整六类 class-source 输出，全部写入隔离目录后以空 classpath/sourcepath 重编，再以 `-Xverify:all` 运行新产物。对原始 leg 执行同一运行方式，并逐字比较 exit/stdout/stderr。JDK8 与 JDK23 冻结 class 输入各走一轮；测试代码使用仓库现有配置选择的 javac/java 工具，JDK 专项重放由 root 验收。
- 新增 `direct_child_array_controls_keep_order_reader_effect_and_type_boundaries`，同样在两条冻结 direct legs 上从 `ownGridDirect` Code 构造不执行的变体：交换 parent index、在 child store 后插入 `dup; pop`、插入 5 字节 `iconst_1; invokestatic mark; pop`、把 DerivedA child `anewarray` 的 CP 项改为 Object。变体只经过 class-source 恢复；断言完整方法 fallback、没有半份 `new Base[][]`。Object[] 变体另外要求拒绝文本保留 `java.lang.Object[]`→`Base[]` 类型冲突。
- 代码 patch helper 通过 class facts 与 method Code span 定位 `ownGridDirect() [[LBase;`，锁定原 Code 长度 47、`max_stack=9`、无 handlers/Code subattributes，并从当前 class 自己查 CP 项。extra-reader 增量为 +2；effect 序列明确为 +5，符合已复核的控制字节长度，不执行任何变义变体。
- 增加缺失 `Mid.class` 的 direct family fallback 控制，以及只对 Main standalone class 建立 SSA/RecoveryRequest 的 method-only unknown-hierarchy fallback 控制。两者都要求完整方法保留 `@bytecode` 且不产出 `new Base[][]`；不会仅由类名推断继承关系。

## 未执行与验收边界

没有编译测试，也没有运行变体、完整原始/恢复家族或任何 JDK。关于 Object[] 变体抵达准确 Builder 类型拒绝、order/reader/effect 变体分别由预期结构门拒绝、method-only 确无 selected hierarchy，以及所有完整 direct source 能重编/运行，当前均是测试要验证的断言，不是实测结论。Helper 的字节锚来自已冻结 `ownGridDirect` 原始 Code 与控制设计记录；JVM 不兼容 Object[] 控制、乱序控制和 interleaved-effect 控制均不执行。

当前测试断言完整 class-source 家族不含正文拒绝标记，所以若 `collectionGridDirect` 或伴随类仍有独立 Signature/source-proof 拒绝，测试会如实失败；没有通过放宽泛型门槛来预先接受。focused `ArrayInitializers` 私有 ownership/提交/预算断言由另一实现者处理，本文件的完整正文 fallback 不能单独证明具体内部拒绝分支。

## Root focused v1 编译反馈

根据 `results/root-focused-integration-v1/stderr` 修复了测试编译问题：从 `jarde_java` 显式导入 `DeclaringClass` 和 `JAVA_8`，并将三个 BCI slice 的迭代器都显式转换为 `.iter()` 后再 `chain`。此次只改本 integration 文件及本说明；没有运行 Cargo、Git、rustfmt 或 Java。修复尚未由后续 focused 运行验证。

root-focused-integration-v2仍exit101：JAVA_8公开于pass模块，root依据lib.rs公开出口修正导入为jarde_java::pass::JAVA_8，原日志保留。尚待v3执行。
