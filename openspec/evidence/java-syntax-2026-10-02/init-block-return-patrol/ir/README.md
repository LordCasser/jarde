# 初始化块 return 抑制取证（`recover-init-block-return-suppression` 实现腿）

主线 `352a87d7` 上完成 tasks 1.1/1.2/2.1/2.2/3.1/3.2 的取证与验收记录。可重放命令均在工作树根。

## 1. 发射点取证（tasks 1.1，第一个取证义务）

**语句发射点**：`crates/jarde-java/src/emit.rs` —— `Emitter::stmt` 的 `StmtKind::Return` 臂；
体级入口是 `Emitter::body`（envelope 之后唯一调用 `stmts` 的地方）。

**语境标识在该点的可得性**：`emit()`/`emit_source_map()` 都携带 `declaration:
Option<&Declaration>`，`crates/jarde-java/src/declaration.rs` 已按成员名分派
`<clinit>` → `DeclarationForm::StaticInitializer`（`<init>` → `Constructor`）。
`body()` 本就沿该分派做**顶层末 return → 闭括号投影**（return 的 origin 锚在 `}` 上）；
抑制因此落在同一分派上：`body()` 设置 `Emitter::initializer`，`stmts()` 在该语境跳过
`Return { value: None }`（void 无值形）。方法/构造器（及其余一切 DeclarationForm）不设
该语境，return 逐字保留。

**根因证据（为何顶层投影漏网）**：javac 生成的 `<clinit>` 尾 `return` 是字节码**顶层**
终结符——[javap/F2.clinit.javap.txt](javap/F2.clinit.javap.txt) `28: return`、
IBRMulti `54: return`、IBRBranch `49: return`，全类唯一且不在分支内
（`IBRInst.clinit.javap.txt` 为空：实例初始化块没有自己的 `<clinit>`，其语句内联进
`<init>`，见变体表）。但区域构建器把
guard 抛错后的 fall-through 尾部结构化为 `else` 臂，终结符随尾部一起**嵌套**进臂内
（实现前输出：`else { …; return; }`）。`body()` 只识别"体自身最后一条语句是 return"的
形态，嵌套形态漏网发射——这就是 F2 的 JLS §8.7 违例。

**其余 `<clinit>` 文本通道核对**（全部逐字不变，除 return 抑制）：

| 通道 | 位置 | 抑制关系 |
| --- | --- | --- |
| `<clinit>` 物理恢复 | `report::recover*` → `emit()` | 语境过滤直接生效 |
| enum switch 组重发射 | `report::emit_class_source_enum_switch_group` → 同一 `emit()`（带 declaration） | 自动继承抑制 |
| enum map 后缀 | `enum_constants.rs::prove_map_initializer_suffix` 选择止于 `loop_statement` | 证明本身排除尾 Return，无变化 |
| 静态字段初始化折叠 | `emit_class_initializer_value`（` = expr` 片段） | 纯表达式，无 return |
| 匿名类成员投影 | `class_source.rs` 排除 `ACC_STATIC` 成员 | `<clinit>` 不可达 |
| enum 构造器用户语句 | `emit_class_enum_constructor_statements` | 构造器通道，return 合法 |

## 2. 变体（tasks 1.2，实现前后已冻结）

[variants/](variants/) 三个 `javac --release 8 -g:none` 可编、`java -Xverify:all` 可跑的变体
（原类行为基线 [results-ir/runs/](results-ir/runs/)）：

| 变体 | 形态 | 实现前 | 实现后 |
| --- | --- | --- | --- |
| `IBRMulti`（多静态块） | 4 个静态块+穿插字段初始化，javac 合并为单 `<clinit>`，尾部整体成 else 臂 | `return;` 在 else 臂内（[before/](results-ir/before/)，重编拒 `返回外部方法`） | 该行移除，其余逐字相同（[after/](results-ir/after/)） |
| `IBRInst`（实例初始化块） | 实例初始化块内联进两个 `<init>`，尾 return 在构造器通道 | 合法且忠实（重编通过） | **逐字节不变**（构造器通道不动） |
| `IBRBranch`（静态块分支 return） | 双 guard 抛错，return 嵌套**两层** else 臂 | 拒编 | 该行移除，其余逐字相同 |

实现前重编拒编记录：[results-ir/recompile/](results-ir/recompile/)
（F2/IBRMulti/IBRBranch 拒、IBRInst 过）。对照腿：固定 JADX dev
（`--no-debug-info`，[jadx/](results-ir/jadx/)）四类全部合法无 return。

## 3. 三方对照（tasks 3.2）

原 class / 固定 JADX 重编 / Jarde 重编（`class-source` 输出 → `javac --release 8 -g:none`）
全部 `java -Xverify:all`，逐路径一致：

```
F2: 5:42        IBRMulti: 3:12:9
IBRInst: 4:8:4:18   IBRBranch: 7:9
```

（[runs/jarde-recompiled.out](results-ir/runs/jarde-recompiled.out)、
[runs/jadx-recompiled.out](results-ir/runs/jadx-recompiled.out)、
[runs/F2.orig.out](results-ir/runs/F2.orig.out)、[runs/variants.orig.out](results-ir/runs/variants.orig.out)；
SHA 见 [results-ir/sha256.txt](results-ir/sha256.txt)，F2.class 与 super-default-patrol 冻结
fixture 同 SHA `1efb8fad…`。）

## 4. 全量回归（tasks 3.1 的 diff 守护）

用主线腿（stash 前后两个 release 二进制）扫 462 个工件（`tests/**` 全部 `.class`/`.jar`
+ 本巡查与 super-default 巡查 fixture，逐类 `class-source`）：**文本输出仅 3 处差异**
（F2/IBRMulti/IBRBranch，各恰好一行 `<clinit>` 内 `return;` 移除），其余 459 工件文本逐字
相同（stderr 差异全部为 `elapsed_millis` 计时）。既有 `<clinit>` 呈现（enum 折叠家族、
接口初始化器投影等）与全部方法/构造器呈现零变化。
