# assert 语句巡查（2026-10-03）——合成降低显式呈现

assert 域巡查（主线 `b90400f7`）。固定转录 [fixture](fixture/)（A1：带消息/无消息 assert + 嵌套类各自 `$assertionsDisabled`；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 orig.out（`10`/`25`/`2`/`AssertionError`）。

## 现状

| 场景 | 主线 Jarde |
| --- | --- |
| assert 降低恢复（`<clinit>` 的 desiredAssertionStatus 行、`if (!$assertionsDisabled)` 守卫、条件或抛） | **完整且行为一致**（零引用；守卫字段语义保真，`-ea`/无 `-ea` 两态对） |
| 源码形状 | 未呈现 `assert cond : msg;`——`$assertionsDisabled` 合成字段声明、`<clinit>` 初始化行、if 守卫三层全部显式 |

## 定性

javac 合成模式（与 `val$x`/`this$0`/`access$NNN` 同族）：`static final boolean $assertionsDisabled`（synthetic，`<clinit>` 内 `desiredAssertionStatus() ? 0 : 1`）+ 使用点 `if (!X.$assertionsDisabled) { if (!cond) throw new AssertionError(msg); }`。可编可运行（非正确性缺口），属源码忠实度/合成物消隐族——折叠域外的独立合成识别。

## 处置方向

`recover-assert-statement-sugar`（中片）：识别该合成模式并回写 `assert cond : msg;`——字段+clinit 行隐藏（synthetic 佐证：名字模式+ACC_SYNTHETIC+初始化形）、守卫块折叠为 assert 语句；识别失败保持现呈现（忠实兜底）。与折叠域正交（无族依赖）。

原 class 为行为基准（含 `-ea` 态）。
