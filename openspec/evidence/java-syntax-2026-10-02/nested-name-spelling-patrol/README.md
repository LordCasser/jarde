# 嵌套类型源码位拼写巡查（2026-10-02）

字符串/枚举 switch 巡查引出（主线 `dc73e881`）——switch 家族本体（数值/字符串 hash 分派/枚举 ordinal 表/fall-through）**全部完整恢复且行为一致**；缺口在外围：嵌套类型引用拼写。固定转录 [fixture](fixture/)（V1.java/V1.class/V1$Op.class/fam.jar，SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 orig.out。

## 表现

单类与 jar 输入一致，`V1$Op`（二进制名）出现在**源码语法位**：

- 参数类型：`public static int enumSwitch(V1$Op arg0, int arg1)`（第 46 行）
- 枚举常量引用：`enumSwitch(V1$Op.MUL, 7)`（第 90 行）

javac 报"找不到符号 变量/类型 V1$Op"——Java 源码嵌套类型须 `Op`（同外围类内简单名）或 `V1.Op`（点分限定），`$` 拼写非法 → **含此类引用的类不可重编**。JVM 字节码层常量池本就是 `V1$Op`，呈现代码按常量池名直拼。

## 判别

- 包级类引用（`Other`）简单名正常（K2 patrol 先例）——缺陷仅嵌套形态。
- enum 折叠家族（枚举类自身文本）不受影响；此为**外围类引用嵌套类型**的呈现拼写。

## 处置方向

`recover-nested-type-source-spelling`（窄呈现切片）：类型引用拼写在源码语法位（声明/参数/局部/调用限定/new/instanceof/cast）呈现时，若常量池名为 `Outer$Inner` 形且 `Outer` 等于当前类名（自嵌套）则呈现简单名 `Inner`；否则呈现点分 `Outer.Inner`。仅改拼写层（类型名→源码名的既有转换处），不动证明。V1 家族（含枚举常量限定 `Op.MUL`）整类可重编；包级简单名、平台限定名逐字不变。

原 class 为行为基准。
