# verification — make-required-conversions-explicit

## 反例（修前，本机复现）

```text
p3-required-conversions/v8 …: the two bodies' observable traces differ:
line 1: original  value castPart(C)Ljava/lang/String; ['A'] -> "65!"
        generated value castPart(C)Ljava/lang/String; ['A'] -> "A!"
```
文本侧同样红：`left: "return \"\" + arg0 + \"!\";"` vs 期望 `"return \"\" + (int) arg0 + \"!\";"`。

## 修复形状

- `ExprKind::Cast { ty, value }`：按 JLS 15.16 unary 位置拼写，origin 继承 value（转换没有自己的指令）。
- `Expr.presented: Option<Type>`：节点自带的部分由形状决定，运行期才知道的（局部、调用返回、字段）在构节点处用 `Expr::presenting` 写入；`None` 表示"本层不陈述类型"。
- `build.rs::meeting_position`：相等→原样；int 常量落在 byte/short/char → 原样（JLS 5.2/5.3）；任一侧引用→原样；JLS 5.1.2 加宽→`Cast`；其余→拒绝并具名 value BCI/两种类型/位置。
- 覆盖位置：concat 片段、调用与构造实参、`return`、声明与赋值、字段写入（含 accessor 写路径）、`iinc` 合成赋值。

## 证据

| 项 | 结果 |
| --- | --- |
| `tests/p3_required_conversions.rs`（8 项，含计费对照） | 8 passed |
| 受控 JDK 编译执行对照（12 成员全部 `Executed`、31 行 trace 两侧一致） | 3 passed（`--ignored`） |
| 全量门禁 | 1506 passed / 0 failed；clippy 0 warning |
| 计费 | 同一请求 `ir_items` 140 → 140、`normalization_clones` 0 → 0，仅文本 +6 字符 |
| 反例（把加宽分支改回原样） | 文本 5 项 + JDK 对照同时变红；还原后全绿 |

## 有意改变的既有断言

1. `tests/p5_bulk_corpus.rs` 的 `MANY_METHOD_CLASS.output_bytes` 25710→25716 与两臂 43945→43951：来自**布尔字段写入**的拼写修复（`FLAG = true;` 原为 `= 1;`，javac 拒绝），与 cast 无关。
2. `fixture` 人口 pin `(51,246,44,125,8)` → `(52,259,44,125,8)`：新增样本类。
3. `corpus-fingerprint.json` 由仓库自带重生成入口更新，diff 只含新增文件。

## 边界（未覆盖形状）

- lambda/方法引用的目标类型（产物里无声明）、引用之间的可赋值性（不做子类型判定）、数值窄化（字节码里必带本层不建模的指令）、算术结果类型不可知（如 `b++`）：各自保持既有行为或按契约拒绝。
- `int→long/float/double` 分支只有单测覆盖（真实可验证字节码不可达）。
