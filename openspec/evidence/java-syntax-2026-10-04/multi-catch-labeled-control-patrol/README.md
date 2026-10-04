# multi-catch 与标签控制流巡查（2026-10-04，root）——验证健康，非缺口

按 Goal "持续巡查各个反编译语法分析模块，构造各种 Java 语法场景，编译，然后对比源码、jadx、jarde" 巡查两个 Java 7+/8 常见控制流形：**multi-catch**（`catch (A | B e)`）与**标签化 break/continue**（含 `continue outer` 被改写为不带标签的 `break` 这一可疑点）。主线 HEAD 二进制（`6cacbdeb` 态）。

**结论：两形均正确恢复、可重编、行为逐字一致。特别地，root 怀疑的"`continue outer` 被译成普通 `break` 会不会改变语义"经对照实验证明是安全的等价改写——但这依赖一个条件（见下），jarde 在条件不成立时会正确发射 `continue loop`。非缺口；本记录目的是避免将来重复巡查，并把这个"看似可疑实则安全"的改写及其成立条件写清楚。**

固定转录见 [fixture](fixture/)（`M1.java`/`M1.class`、`C.java`/`C.class`、`M1-original.out`）与 [results](results/)（两份渲染全文、两份渲染源、两份原 class 基线）。

## 探针一（[fixture/M1.java](fixture/M1.java)）：multi-catch + 标签 break/continue + 标签 break switch

原 class 基线（[results/M1-original.out](results/M1-original.out)）：`42/-1/-1` / `7` / `ab`。

| 形 | 源码 | jarde 呈现 | 判定 |
| --- | --- | --- | --- |
| multi-catch | `catch (NumberFormatException \| NullPointerException e) { return -1; }` | `catch (java.lang.NumberFormatException \| java.lang.NullPointerException local1) { return -1; }` | ✓ 联合类型正确恢复为 `A \| B` |
| multi-catch 后接普通 catch | `catch (Exception e) { return -2; }` | 同形恢复 | ✓ |
| 标签 continue + 标签 break | `outer: for … { for … { if (j==1) continue outer; if (i==2) break outer; } }` | `loop: for (…)` + `break;`（continue outer）+ `break loop;`（break outer） | ✓ 行为一致（见下分析） |
| 标签 break 跳出 switch | `loop: for … { switch(i){ case 1: … break loop; } }` | `loop: for (…)` + `case 1: … break loop;` | ✓ |

整类渲染 **quotes=0、not recovered=0**，重编 `javac --release 8` **exit 0**，`java -Xverify:all` 输出与原 class **逐行一致**（`42/-1/-1`/`7`/`ab`）。

## `continue outer` → 普通 `break` 的可疑点及其安全性（本巡查的主要产出）

`labeled()` 中源码 `continue outer`（内层循环满足条件时跳到外层下一轮）被 jarde 呈现为**不带标签的 `break`**（跳出内层循环）。二者**只在"内层循环是外层体的最后一条语句"时等价**——因为此时跳出内层后外层体也随即结束，控制流自然回到外层的更新表达式，效果等同 `continue outer`。

**这是安全的等价改写，不是语义偏差**：`labeled()` 里内层 `for` 确实是外层 `for` 体的最后一条语句（其后只有 `return 7;` 在循环外），重编运行输出 `7` 与原 class 一致。

**但 root 不接受"看起来对"，构造了条件不成立的对照实验**（[fixture/C.java](fixture/C.java)）：把 `continue outer` 放在"内层循环**后面还有语句**"的形里——此时若 jarde 仍译成普通 `break`，就会**多执行**内层之后的语句，是静默行为偏离。

### 对照实验（[fixture/C.java](fixture/C.java)）

```java
outer: for (int i = 0; i < 2; i++) {
    for (int j = 0; j < 2; j++) {
        if (j == 1) continue outer;
        sb.append("in" + i + j + ";");
    }
    sb.append("after" + i + ";");   // ← 若 continue outer 被错译为 break，此行会被执行
}
```

原 class 基线（[results/C-original.out](results/C-original.out)）：`trail=in00;in10;`（**不含 `after`**）/ `trailBreak=in00;`。这是判别性可观察点——`break outer` 版（`trailBreak`）也不含 `after`，而错译的 `continue`→`break` 版会含 `after`。

jarde 呈现（[results/C-rendered-source.java](results/C-rendered-source.java)）：

```java
loop: for (local1 = 0; local1 < 2; local1 = local1 + 1) {
    local2 = 0;
    while (local2 < 2) {
        if (local2 == 1) {
            continue loop;          // ← 带标签的 continue，未退化为 break
        } else { … local2 = local2 + 1; }
    }
    local0.append("after" + local1 + ";");
}
```

即当内层循环后有语句时，jarde **正确发射带标签的 `continue loop`**（而非退化的 `break`）。重编 `javac --release 8` exit 0，运行输出与原 class **逐字一致**（`trail=in00;in10;` / `trailBreak=in00;`）。

**结论**：`continue outer` 的呈现是**依控制流形状而定**的——内层循环是外层体末条时退化为等价的 `break`（更简洁），否则保留带标签的 `continue`。两条路径都语义正确，无静默偏离。`trailBreak` 版（源码本就是 `break outer`）呈现为 `break loop`，也一致。

## 归属（查重，非新缺口）

- **multi-catch**：inventory **CF-15** 状态"部分已测，普通 catch/multi-catch 首片未见差距"；已有 `recover-typed-catch-boundary-return`（其正例即 `PlainMultiCatch.choose`）。本巡查的多类型联合 + 后接普通 catch 形是对 CF-15 的一个**新正例数据点**（行为一致），非新差距。
- **标签 break/continue**：inventory **CF-09**（"循环出口的 break、continue、标签与嵌套出口恢复；保持退出到哪一层循环"）与 **CF-11**（嵌套循环区域）覆盖；已有 `recover-labeled-loop-tail-coverage`。本巡查的 `continue outer` 末条退化 + 非末条保标签两形是 CF-09 的**新正例数据点**。

## 处置

**不立 spec**（两形均正确，无差距）。价值有二：(1) 把"标签控制流在多语句外层体下正确保留 `continue loop`"这一非平凡行为钉为可复现基线，将来若某片改动 region/latch 逻辑导致 `continue` 退化为错误的 `break`，本 fixture 能作为回归探针；(2) 澄清 `continue outer`→`break` 的退化是**有条件的安全等价**而非 bug，避免将来巡查者把它误报为语义偏差。

**呈现观察（非缺陷，登记）**：`labeled()` 渲染文本中 `break loop;` 之后那个闭合花括号**缩进异常**——[results/M1-rendered-source.java](results/M1-rendered-source.java) 第 26 行的 `}` 缩进为 4 空格，而它闭合的是第 24 行 `} else if (local0 == 2) {` 打开的块（同级兄弟行为 16/12/8 空格）。root 核实花括号**总数平衡（15/15）且配对正确**（26 闭 else-if、27 闭内层 for、28 闭外层 for、30 闭方法），`javac` exit 0、行为逐字一致——故属**纯缩进呈现瑕疵**，不影响可编译性与语义，疑与 `recover-labeled-loop-tail-coverage` 域的 tail 覆盖发射相关。未立项（纯呈现，优先级低）。

原 class 为行为基准（`42/-1/-1`/`7`/`ab`；`trail=in00;in10;`/`trailBreak=in00;`）。
