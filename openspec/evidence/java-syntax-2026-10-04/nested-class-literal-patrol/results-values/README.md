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
