# Comparator 家族巡查（2026-10-05 root）——critical 第 18 锚 + **第 6 诊断族**

## critical 第 18 锚（新族：匿名类实参引用转换）：`Collections.sort(c, new Comparator<User>(){...})` 整条语句吞掉

byAnon（**匿名 Comparator 实参——pre-lambda 排序最常见形**）：sort 调用语句**被完全吞掉**，幸存 `new ArrayList(copy); return c;`——**隔离编译 exit 0、渲染 `[30, 10, 20]` vs 原 `[10, 20, 30]`**（可编译错：排序静默丢失，第一不变量违反）。

诊断文本：**"the parameter 1 of the invocation at BCI 17 is declared \`java.util.Comparator\` presents \`CP$1\` but the invocation requires \`java.util.Comparator\` and this layer has no safe reference conversion evidence"**——**第 6 诊断族（匿名类实参的池形呈现 vs 声明参数类型的引用转换证据缺失）**。伴生 `generic_source_shape_unproved`（类名无可拼源形）为类级注记（池形名债务域）。

## 健康面（负结果）

- **byNameAge**：`Comparator.comparing(User::getName).thenComparing(User::getAge).reversed()` **全链恢复**（方法引用 companion 内联+cast 链+默认方法链式调用）；
- **byLen**：`Arrays.sort(String[], lambda)` 双参 lambda 内联恢复；
- **CP$1 伴生类**本身 quotes=0（compare 体恢复）——失败只在**宿主侧实参转换**；
- 行为 `[bo:30, al:40, al:20]/[al:20, bo:30, al:40]/[a, bb, ccc]` 一致（byNameAge/byLen）。

## 边界判别补完（同日晚，AN/AD 矩阵）——第 6 族触发条件钉死

- **四位置全恢复**（return/实参/字段初始化/局部变量，own 非泛型接口 `AN$Op`）：实参位渲染 `(AN$Op) new AN$2()` cast——**own（same-run）类参数位有转换证据**；
- **泛型 own 方法实参也恢复**（`runGen((Object) new AD$1(), …)`——泛型性无关）；
- **JDK 参数位双失败**：`new Thread(new Runnable(){...})` ctor 实参与 `Collections.sort(c, new Comparator<String>(){...})`（CP 复刻）同族命中（"declared \`Runnable\` presents \`AD$2\` … no safe reference conversion evidence"）；
- **viaThread（同族 JDK ctor 位，SAFE）**：幸存 `new SB(); try{}catch(InterruptedException){} return sb.toString();`——**隔离编译 exit 1（checked 异常 catch 于空 try 体=编译错误）= 第一不变量不违反**；[隔离实证](results/isolated-viaThread-wrong.java)——记录为可恢复性缺口（吞掉的 Thread/匿名/start/join 应可恢复），**非 critical 锚**（首记"可编译"为本人笔误，同日自纠）；级联伴随两行 local2 reads refused；
- **结论**：第 6 族边界 = **匿名类实参的目标参数类型是否 same-run 可证**——own 接口有 InnerClasses/实现链证据（cast 呈现），JDK 类型无证据（整调用吞掉）。

## 处置

soundness spec 补第 6 族锚场景；census 记第 7 条；账本 **18 锚/6 族**。

## 处置（2026-10-06，change `recover-platform-interface-argument-widening`）

第 6 族的实参侧关闭：`snapshot_header_chain_widens_with` 走到目标名时**先判命中、后取目标
header**——`CP$1` 自己的 class-file header 逐字列出的 `java/util/Comparator` 就是单边证明，
平台的 header 不必在快照里（源侧 header 链仍逐级完整证明；不查 classpath）。

- `cp.jar` 重渲染（同一冻结字节、同一入口）：[`results/jarde-CP-after-platform-interface-widening.txt`](results/jarde-CP-after-platform-interface-widening.txt)
  ——`refusals = 0`，`CP.byAnon` 写出整条 `java.util.Collections.sort((java.util.List) local1, (java.util.Comparator) new CP$1());`；
- 本巡查的隔离判别形（`[30, 10, 20]` vs `[10, 20, 30]`）以可编译源 `IS` 冻结在
  [`tests/fixtures/recover-platform-interface-argument-widening/IS.java`](../../../../tests/fixtures/recover-platform-interface-argument-widening/IS.java)，
  剥离、编译（installed javac `--release 8` + 真 javac 8）并运行回答 `[10, 20, 30]`
  （[`tests/recover_platform_interface_argument_widening.rs`](../../../../tests/recover_platform_interface_argument_widening.rs)，ignored replay）；
- `AN` own-interface 四位逐字节零回退（同一测试钉住）；
- 同源 `AD` 实测 `refusals = 0`：[`results/jarde-AD-after-platform-interface-widening.txt`](results/jarde-AD-after-platform-interface-widening.txt)
  ——`new java.lang.Thread((java.lang.Runnable) new AD$2(local1, arg0))` 与随后的 `start()`/`join()`
  一并恢复，巡查记录的 viaThread 可恢复性缺口（此前孤立渲染吞掉 Thread/匿名/start/join）随之关闭；
  `viaSort`/`viaGeneric` 同族位一并呈现。
