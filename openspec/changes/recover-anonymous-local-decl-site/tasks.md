## 1. 取证与冻结

- [ ] 1.1 以 `anonymous-super-args/AnonymousSuperArgs$1` 为锚重放现状：完整源集 `javac --release 8` 退出 1、`main` 分配点的 verbatim 呈现、原 class `java -Xverify:all` 事件日志（`openspec/evidence/java-syntax-2026-09-27/anonymous-super-args/` 既有基线之上补本片前后对照）。**须先读 [design.md](design.md) 的五处判据**（尤其判据 5 的接口路径显式遏制），并在取证中确认 root 的实测事实：根方法 `main` 返回 `void`、分配点在方法体**中部**（其后有两条 `println`）、局部 `instance` **随后被读取**（`instance.render()`）、fixture 以 `-g:none` 冻结故 class **无 `LocalVariableTable`**（`javap -l` 计数 0）。
- [ ] 1.2 冻结正负例：局部声明初始化形正例（`AnonymousSuperArgs$1` 本身）；声明类型不可重拼（父类 binary 名含 `$`，即嵌套父类）负例；初始化值非唯一分配点负例；**同一方法内两个声明初始化位各含一个分配点**负例（design 判据 2 的"分配点唯一性"不变量）；**局部被后续读取且该读取在父类上不可解析**负例（design 判据 3 第三项）。
- [ ] 1.3 **双腿调试信息对照**（design 风险 3）：另冻结一个以 `-g` 编译的**同形**对照（含 `LocalVariableTable`），确认实现后两者的左端类型来源都是 `new` owner 而非 LVT——即实现**不得依赖调试信息**。记录两者的呈现与拒绝原因。
- [ ] 1.4 **共享站点扫描的遏制负例**（design 判据 5，**不可省略**）：冻结一个类，其某方法在**局部声明初始化位**含一个已证的匿名**接口**分配点（`I x = new I() { … };`），且该类当前**不投影**。记录其放宽前的呈现（作为逐字节对照基线）。理由：`class_source_direct_return_new` → `class_source_anonymous_return_site` → `_anonymous_return_sites` 是**接口路径与父类路径共享**的（接口投影在 `facade.rs:3293` 解构该向量、再于 3367 委派父类投影），故放宽站点形会顺带激活接口匿名投影——那是本片 Non-Goals 之外、无人取证的能力。

## 2. 实现

- [ ] 2.1 站点扫描接受局部声明初始化形（分配表达式为唯一初始化值），**且覆盖分配点位于方法体中部、其后仍有语句的情形**；保持直返形与 `recover-anonymous-mixed-super-capture` 的判据逐字不变（其新锚 `anonymous-super-mixed-direct` 必须仍逐字节相同）。**不变量**：该方法内已证分配点计数仍须为 1；分配点**之后**的语句必须整体 `quality=structured` 且无引注，否则整方法回落物理文本（不得只投影前半段）。
- [ ] 2.2 emitter 增加赋值左端声明类型重拼（匿名子类名 → 父类源码名），不可证明时保持物理文本。**须实现 design 判据 3 的三项检查**：(i) 重拼后的类型可拼写（复用既有 `anonymous_super_source_type_unproved` 判据，父类 binary 名含 `$` 即拒绝）；(ii) 池形类型名结构反射陷阱判据适用（最终文本含 `$` 且被 `STRUCTURAL_REFLECTION_METHODS` 之一消费则拒绝）；(iii) 该局部被后续读取时，若任一读取点在重拼类型上不可解析则**回落物理文本**（不得用 `var`/`Object` 兜底——兜底会改变静态解析结果，属禁止的静默偏离）。在代码注释中保留 design 判据 3 的健全性论证要点。
- [ ] 2.3 `project_class_source_anonymous_super` 根方法门按 **design 判据 1** 放宽：父类源码名的信息来源改为"声明初始化值 `new` 操作数的 owner 类型"，**不得保留对根方法返回描述符的检查**（否则 `void main` 锚永远被拒），也不得改为"忽略返回类型"（会放宽到未取证范围）。捕获证明与参数角色划分复用 `recover-anonymous-mixed-super-capture` 的通道，不新增第二套机制。
- [ ] 2.4 确认 design 的 Open Question：锚的 `main` 在放宽后是否仍全程 `structured`、有无与 `explanation_only`/混合 fallback 冲突。若出现混合 fallback，**停手报告**，不得自行放宽 `present-proved-java-structure` 5.3 的该项要求。
- [ ] 2.5 **按 design 判据 5 的第 1 种方案实现显式遏制**：站点元组增加判别位（`AnonymousSiteShape::{DirectReturn, LocalDeclInitializer}`，由 `class_source_direct_return_new` 的调用方按语句位置给出）；**接口路径的前置改为要求该位为 `DirectReturn`**（与放宽前逐字节等价），父类路径接受两位。**不得**走方案 2（第二套站点派生，违反"不新增第二套机制"）或方案 3（顺带取证接口路径，范围翻倍且与 `recover-proved-anonymous-local-capture` 重叠）。
  - 必须的回归断言：1.4 冻结的遏制负例在实现后**呈现逐字节相同**（仍不投影）；`recover-proved-anonymous-local-capture`(6/6)、`recover-proved-anonymous-inner-this`(8/8)、`anonymous-interface-basic`(DT-05) 全部既有测试逐字通过；corpus 双腿扫描中**接口匿名形零差异**。
  - **停下报告条件**：若 corpus 双腿扫描出现接口路径的任何差异，即为遏制失效，不得以"看起来正确"放行。

## 3. 验收

- [ ] 3.1 `anonymous-super-args` 完整源集 `javac --release 8` 通过、`java -Xverify:all` 事件日志逐行一致；全部既有匿名正负例（含 `anonymous-super-mixed-direct` 与六个 mixed refusals）零回退；`-g` 对照腿同样通过且左端类型来源一致。
- [ ] 3.2 门禁：fmt、clippy **从 `.github/workflows/ci.yml` 46–76 行逐字生成**（含 `--all-features`、29 项 `-A`）、`openspec validate --all --strict`、corpus 双腿扫描（差异应仅赋值初始化形；**若出现第 2 个差异类即越界信号，停下报告**）、全仓测试（当前主线基线 **296 目标 / 2937 passed**）、`git diff --check`。磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean；**同一时刻只允许一个 cargo target 存在**。
- [ ] 3.3 root 独立复核五处判据的实现（尤其判据 5 的接口路径显式遏制是否成立：遏制负例逐字节不变 + 接口匿名形 corpus 零差异）、左端重拼的健全性、双腿调试信息对照、三方行为与 `anonymous-super-args` 的可编译性转变，更新 DT-06 账本与 `present-proved-java-structure` 5.3 剩余范围说明。（留 root）
