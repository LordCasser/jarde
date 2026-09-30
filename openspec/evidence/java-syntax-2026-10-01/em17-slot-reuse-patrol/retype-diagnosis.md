# 诊断记录：数组槽复用跨元素类型的分段声明（recover-array-slot-retype-locals）

基线 `1c920ca1`（mainline HEAD），隔离 worktree。fixture SHA 复核（[fixture-sha256.txt](results/fixture-sha256.txt)）：
`A1.class` = `24a58f9a…ced0`、`A2.class` = `0e441de3…e335`，逐字一致；基线恢复文本
[A2-retype.before.java](results/A2-retype.before.java)（= 既有 `results/A2.jarde.java`，SHA
`6844af18…b37d`）与 [results/A1.jarde.java](results/A1.jarde.java) 逐字重放一致。

## 1. A2.fillCalc 槽 2 的段事实（SSA def 归属）

字节码（`javap -c -p -l A2.class`；`-g:none`，无 LVT/行号表）：

- 段一 def：`astore_2` @35 —— 槽 0 的 `int[]`（@0–25 构造，`astore_0` @25）经 `aload_0`@34
  拷贝入槽 2（增强 for 的隐藏数组副本 lowering）。array 通道拼写 `(int, 1)`。
  段一读取：@36（`arraylength`，循环边界）、@48（`iaload` 的数组操作数，循环体元素读）。
- 段二 def：`astore_2` @75 —— `iconst_3; newarray boolean`@72–73 直接写入。array 通道拼写
  `(boolean, 1)`。段二读取：@76（`bastore` 的数组操作数）、@98（`baload` 的数组操作数，return 内
  `f[1]`）。
- 两段读取集不相交：每条 `aload_2` 的 SSA 读值的 def 恰为一条 store 指令值（@36/@48 → @35；
  @76/@98 → @75），槽 2 上无任何 phi（循环体内不再写槽 2，回边无合并）；两类拼写经
  `build::array_of_value`（帧描述符 + `newarray` 操作码既有通道）取得，无新证明事实。

基线呈现把槽 2 拼成单变量 `int[] local2`（首个 def 赢得 `decide_types`），段二存储呈现
`local2 = new boolean[3];`，`javac --release 8` 报两处错误：

```
A2.java:31: 错误: 不兼容的类型: boolean[]无法转换为int[]
A2.java:32: 错误: 不兼容的类型: boolean无法转换为int
```

## 2. 呈现形态：段内新名（编译事实裁决）

Java 源码不允许同作用域同名重复声明（`javac --release 8` 探针：`已在方法 fillCalc()中定义了
变量 local2`），故段二以新名呈现。命名沿用 NameTable 既有路径：`SlotEvidence::Split` 的无名
变量按序发明 `local{slot}`，同名冲突走既有碰撞后缀 → 段一 `local2`、段二 `local2_2`
（`names.rs` 既有 `local{slot}` + `_2` 规则，非本次新造）。段二声明落点 = 既有就地声明规则
（其全部使用与首个存储同区域 → 首个存储处 `boolean[] local2_2 = new boolean[3];`）；段一使用
横跨 for 区域 → 维持既有提升声明 `int[] local2;` 于方法顶。拷贝别名 `local2 = local0;` 无独立
消除机制，按分段呈现自然保留（design 决策 3）。

## 3. 实现落点

- `crates/jarde-java/src/reuse.rs`：新增第三条窄拆分证明 `array_retype_split`（槽非参数/守卫头/
  无 debug 记录、无不可达块；≥2 写且各写均经 array 通道拼出完整数组型、至少两拼写不同；每条读
  值的 SSA def 唯一归属一条写——非平凡 phi/Caught/Entry 读一律拒绝；任一 def 的值在下一写 BCI
  及之后仍有使用拒绝——栈上存活/合流形状保持单变量）。段边界 = 定义 store 序，每写一段。
- `crates/jarde-java/src/build.rs`：`array_of_value` 提为 `pub(crate)`（同一读取通道供 reuse 使用）。
- `crates/jarde-java/src/report.rs`：`reuse::plan` 传入 `&operations`。
- 下游零改动：`slot_uses`/`decide_types`/声明提升与就地声明/命名/渲染均按 `LocalVariable`
  （slot, index）既有路径自然分段。

## 4. 变体边界（前后输出见 [variants/](variants/)，冻结 class 见 `tests/fixtures/p3-array-slot-retype-locals/v8/`）

| 变体 | 前 | 后 |
| --- | --- | --- |
| A2.fillCalc（int[]→boolean[] 两段） | `local2 = new boolean[3];` 不可编译 | `boolean[] local2_2 = new boolean[3];`，整类 javac 8 通过，行为 `2,3,4,true` 一致 |
| V1.three（int[]→boolean[]→Object[] 三段） | `local1 = new boolean[3];`、`local1 = new Object[]{…}` 不可编译 | `local1`/`local1_2`/`local1_3` 三段自有声明，javac 8 通过，行为 `2,3,4,truey` 一致 |
| V2.sameType（同槽两个 int[] def） | 完整可编译 | **逐字不变**（SHA 前后同为 `a8a2f7df…c4ed`） |
| V3.join（if/else 双 def 在汇合点 phi 合流，真别名） | 维持既有单变量呈现 | **逐字不变**（SHA `959dce5c…83f6`；既有呈现本就不可编译，属"不可分段形状保持既有呈现"） |
| V4.loop（循环携带 phi 合流） | 同上 | **逐字不变**（SHA `2a275195…9074`） |
| A1（动态维度/混合初始化器对照） | 完整恢复 | **逐字不变**（`results/A1-retype.after.java` = 既有记录，SHA `ca9109e9…ba1a2`） |

## 5. 回归测试

`tests/p3_array_slot_retype_locals.rs` + 冻结 fixture `tests/fixtures/p3-array-slot-retype-locals/`
（v8 六个 class + baseline 前文本 + expected stdout + SHA）。已验证：撤掉 reuse.rs/report.rs/
build.rs 修改后 `the_retyped_slot_presents_one_declaration_per_definition_and_recompiles` 在
`boolean[] local2_2` 断言处失败（旧行为无分段）；修改后六用例全绿。
