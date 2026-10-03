# `recover-nested-class-literal-values` — 实现结果（2026-10-04，coder 切片）

判据落点：`crates/jarde-java/src/decode.rs` 的 `source_internal_name`（`class_literal_type` 的唯一门，全仓仅数组元素名/对象名两处调用）。`!name.contains('$')` 一票否决改为「按 `/` 与 `$` 分段后每段均为合法 Java 标识符」——本地类尾段（`LC$1Local`）、匿名类尾段（`LC$1`）以数字开头被同一判据自然排除；呈现拼写零新增（emit `put_type` → `names::nested_member_reference_spelling`，`InnerClasses` 行集确证）；身份缝（`spell_reference` 池形）不动。

## 冻结输入（本目录 fixture-sha256.txt）

- `fixture/A10.class`、A9 全家族 class + `a9.jar`、`a11.jar`、`A9.orig.out`（原类运行基线）。
- `fixture-variants/WC1/`：字面名含 `$` 的**顶层类**（`WC1$Top.class` 自引用字面量）——判据准入、无 InnerClasses 行 → 两侧保持池形 `WC1$Top.class`，可编译、运行逐字一致。
- `fixture-variants/WV1/`：嵌套类**数组形** `Nested[].class`（`elem`/`array`/`arrayLen` 三形）+ 折叠域形。
- 既有冻结输入不变：A10.java / A11 / A12 / N2 / LC（SHA 见 `results/fixture-sha256.txt`）。

## 引注计数（before = d4bf6325 基线二进制，after = 本片）

| fixture | before `@bytecode` | after | 说明 |
| --- | --- | --- | --- |
| A12 | 6 | **0** | `nestedLit`/`nestedRecv` 两形恢复 |
| N2 | 10 | **0** | 多段/中层/链式 receiver 三形恢复 |
| A11 | 15 | **0** | 反射读注解三形恢复 |
| A9 | 17 | **0** | 与 proposal「A9.main 17 处引注」吻合 |
| A10 | 0 | 0 | 五形逐字不变；`A10.after.txt` 与巡查 `results/A10.txt` **字节一致**（`A10.after-vs-patrol.diff` 为空） |
| LC | 2 | **2** | 负例保持拒绝（本地类字面量 BCI 0 同一引注） |
| WC1 | 3 | **0** | 字面 `$` 顶层类按池形呈现 |
| WV1 | 10 | **0** | 数组形 `WV1$Nested[].class` 准入 |

## 三方对照（原 class / 固定 JADX dev / Jarde 重编，`java -Xverify:all`）

JADX dev 以 `defpackage` 重打包，凡输出暴露二进制名的路径（`getName`）JADX 自身即偏离原类（`jadx-a10.out` 打 `defpackage.A10`、`14`），故三方「逐路径一致」按下列实测口径记录：

| 锚 | 原 class | Jarde 重编 | JADX dev |
| --- | --- | --- | --- |
| A12（`o4.out`） | `A12`/`Nested` | **逐字一致**（`a12-jarde-recompile.out`） | 一致（无二进制名路径） |
| A11（`o3.out`） | `real`/`real`/`true` | **逐字一致**（root+`A11$Tag` 单元工程编译） | 一致 |
| A9（`A9.orig.out`） | `m:7:[a, b]`/`true`/`d`/`2` | **逐字一致**（root+`A9$Meta` 单元工程编译） | 一致 |
| A10（`o2.out`） | 五形 | **逐字一致** | `getName` 路径打 `defpackage.A10`（JADX 重打包所致） |
| WC1 | `WC1$Top` | **逐字一致** | 一致 |
| WV1 数组两形 | `[LWV1$Nested;`/`13` | **逐字一致** | 重打包偏离（`[Ldefpackage.…`/`24`） |
| WV1 `elem`（`getSimpleName`） | `Nested` | **偏离**（打 `WV1$Nested`，见遗留 2） | 一致 |
| N2 `multiLevel`（`getSimpleName`） | `Leaf` | **偏离**（打 `N2$Outer$Mid$Leaf`，见遗留 1） | 一致 |
| N2 `midLevel`（`getName`） | `N2$Outer$Mid` | **逐字一致** | 重打包偏离 |
| N2 `recvChain`（`getEnclosingClass`） | `Mid` | **偏离**（`getEnclosingClass()` 返回 null → NPE，见遗留 1） | 一致（JADX 递归内联成员类） |

## corpus 双腿扫描（465 类）

`tests/fixtures/**/*.class` 全量，`--policy single-class`，基线二进制 vs 本片二进制逐类字节比较：**465/465 逐字同，0 差异**（`corpus-double-leg.diff` 为空）。差异面确实仅类字面量准入路径，冻结 corpus 无此类输入。

## 遗留（单列，不在本片放宽）

1. **折叠深度 ≥ 2 的结构反射形**：jarde 成员折叠为单层（folded child 自身 `member_family.state = "absent"`，N2$Outer$Mid/Led 以平铺 `$` 单元呈现）。平铺单元不带 `InnerClasses` 行，javac 亦不从 `$` 名推断嵌套，故 `getSimpleName`/`getEnclosingClass` 等结构反射在平铺单元上返回池名形/null。`getName` 形不受影响（二进制名保留）。JADX dev 递归内联成员类所以全绿。属成员折叠深度域，非本片准入判据；A12/A11/A9 折叠覆盖到的一层成员全部逐字一致。
2. **折叠 token-tie 锚不认数组类常量**：根文本含自身成员的**数组类字面量**（`Nested[].class`，池名 `[LChild;`）时，静态折叠的重拼锚检查（facade.rs `project_class_source_member_fold` 路径："static fold token … is not tied to one proved class reference"）只认 `Class` 名 == 成员二进制名与 FieldRef/MethodRef owner，不认数组描述符的元素名 → 折叠拒绝 → WV1 退回分离呈现（遗留 1 的 `getSimpleName` 偏移因此放大到一层成员）。基线上该形态方法整体拒绝、根文本无 token，折叠"成功"是平凡情形；本片准入后首次暴露。属折叠域，未顺手放宽。
3. 方法级恢复本身不受上述影响：所有锚方法引注清零、整类/整工程 `javac --release 8` 通过。

## 修正轮（2026-10-04，root 验收驳回后的守卫）

root 验收驳回上表的 N2 行为：基线响亮失败（10 引注、不可编译），切片后变为**可编译且行为不同**（`multiLevel` 静默偏离、`recvChain` NPE 崩溃）——违反消隐/呈现不变量。修正如下。

### 判据（root 修正版，本人独立复核后实现）

> 类字面量若 (a) 其**最终呈现文本仍是池形**（含 `$`，折叠投影没有把它拼成源码嵌套形），且 (b) 其值被 `java/lang/Class` 的**结构反射方法**（`getSimpleName`、`getEnclosingClass`、`getCanonicalName`、`getDeclaringClass`、`isMemberClass`、`isLocalClass`、`isAnonymousClass`、`getNestHost`、`getNestMembers`、`getEnclosingConstructor`、`getEnclosingMethod`）消费 —— 则保持拒绝（响亮失败）。`getName`/注解读取等不在集合内（池形下恰正确，N2 `midLevel` 与 A11 全链实证）。

**本人取证对 root 机制解释的独立复核结论**：`names.rs:204-207` 自嵌套规则（"only a fold projection may spell it as source nesting"）+ 折叠单层——**确认无误**（run 级 artifact 实证：A12 折叠形的 run 级文本是池形 `A12$Nested.class`，最终文本的 `Nested.class` 是折叠投影 emit 后的词法重写）。两个修正（均已与 root 结论兼容地并入实现）：

1. **"最终呈现池形"在 emit 层不可判定**：emit 时刻 A12（折叠后 simple）与 N2-Leaf（永远池形）同为池形，拼写函数返回值区分不了二者——root 建议的 build.rs 返回值判据会误杀 A12。实现改为**两段式**：build 层守卫用"行集内（真嵌套）∧ 非直属成员"确定性拒绝（深层成员在任何呈现下都是池形，无近似）；直属成员由调用方旗标 `RecoveryRequest::with_pool_spelled_members` 决定（standalone-CLASS 根 ⇒ 无折叠 ⇒ 池形保证 ⇒ 拒绝；折叠 road 默认 false，A12 折叠形不受影响）。
2. **判据需补"真嵌套"条件**：`WC1$Top`（顶层 `$` 名）自身无 `InnerClasses` 行，其池形呈现是**忠实的**（原类就是顶层，`getSimpleName` 一致）——按"最终池形"字面判据会误杀。行集成员资格（`declaring_class().inner_class_members()` 含该名）即"真嵌套"证据（javac 只为被引用的嵌套类写行；WC1 实证 0 行）。

### 守卫落点

- `crates/jarde-java/src/build.rs`：`STRUCTURAL_REFLECTION_METHODS` 常量 + `structural_reflection_over_pool_spelled_literal`（call 构造的 receiver 渲染后拒绝）；`Inputs/Builder` 增 `nested_class_members`（行集，来自 `MethodFacts.declaring_class().inner_class_members()`——与呈现同源）与 `pool_spelled_members`。
- `crates/jarde-java/src/report.rs`：`RecoveryRequest.pool_spelled_members` + `with_pool_spelled_members`；recover 传行集与旗标进 build。
- `src/facade.rs`：`recovery_from_with_class_candidates` 在 standalone-CLASS 根置旗标；**遗留 2 就此闭环**——折叠 token-tie 锚补认数组描述符元素名（`[LChild;` 命中元素二进制名），WV1 折叠恢复、三线全部简单拼写且重编运行**三线逐字一致**（`wv1-jarde-recompile-guard.out`）。

### 守卫后对照（`*.after-guard.txt`）

| 锚 | 切片（守卫前） | 守卫后 | 原类 |
| --- | --- | --- | --- |
| N2 `multiLevel`（`getSimpleName`） | 可编译、偏离 | **拒绝**（引注 "…reads `getSimpleName` … pool's form…"） | `Leaf` |
| N2 `recvChain`（`getEnclosingClass`） | 可编译、NPE 崩溃 | **拒绝**（同上） | `Mid` |
| N2 `midLevel`（`getName`） | 恢复 | **恢复**（`N2$Outer$Mid`，逐字） | `N2$Outer$Mid` |
| A12 jar 两形 | 恢复（simple） | **逐字不变**（`A12.zero-regression.diff` 空） | `A12`/`Nested` |
| A11 三形 | 恢复 | **逐字不变**（`A11.zero-regression.diff` 空） | `real`/`real`/`true` |
| A12 standalone `nestedLit` | 可编译、偏离 | **拒绝**（旗标路；`A12-standalone.after-guard.txt`） | `Nested` |
| A12 standalone `nestedRecv`（`getName`） | 恢复（池形） | **恢复**（`A12$Nested.class.getName()`，池形下恰正确） | `A12$Nested` |
| A12 standalone `topLevel` | 恢复 | **恢复**（顶层无 `$`，不涉守卫） | `A12` |
| WV1 三线 | 数组两形一致、`elem` 偏离 | **折叠恢复，三线逐字一致**（遗留 2 闭环） | `Nested`/`[LWV1$Nested;`/`13` |
| WC1 | 恢复（池形） | **恢复**（无行 ⇒ 忠实池形，判据修正 2 保它） | `WC1$Top` |
| A10/LC | 一致/负例 | **不变** | — |

拒绝文本与基线不同（基线是 decode 层 "not part of the provable subset"，守卫是 call 层对结构反射的具名拒绝），响亮性质一致：整方法引注、方法体不发布、不可编译。已按 root 允许的"明确说明"口径记录。

### corpus 复扫（守卫后）

465 类双腿（基线 vs 守卫二进制，single-class）：**仍 0 差异**——守卫与旗标在冻结 corpus 无命中（corpus 无成员类字面量输入），准入判据本身未动。
