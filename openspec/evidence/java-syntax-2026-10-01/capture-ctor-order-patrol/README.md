# 捕获型构造器的 super 序呈现缺陷（2026-10-01 巡查）

内部类/匿名类域巡查（主线 `e9e48aa5`）。固定转录 [fixture](fixture/)（C1 捕获型匿名类 ×2、C2 成员内部类，SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）。

## 发现：javac 合成 ctor 的"先字段后 super"字节码序被逐字呈现为非法 Java

javac 8 对捕获型/内部类构造器发出的字节码把**合成字段的 putfield 放在 super 调用之前**（JVM 层合法；源码层 Java 要求 super() 为 ctor 首句）：

- `C1$1`（匿名捕获）：`aload_0; iload_1; putfield val$base; aload_0; invokespecial Object.<init>` → Jarde 呈现 `this.val$base = arg1; super(); …` —— javac 报"灵活构造器是预览功能"（Java 8 非法）。
- `C2$Inner`（成员内部类 this$0）：`putfield this$0; invokespecial Object.<init>; putfield tag` → 同样 `this.this$0 = arg1; super(); this.tag = arg2;` 非法。

两模式（`val$x` 捕获字段、`this$N` 外部引用）覆盖**所有**捕获型匿名/局部类与所有非静态成员内部类——高频形态；family 分离呈现本身健康（行为对照以 family 联编验证）。

## 根因

ctor 语句呈现按字节码序逐字输出；对"super 前的合成字段存"无规范化。JADX 对同形态按 javac 模式知识重排（super 提前）。正确所有者：ctor 呈现层对**可识别的 javac 合成模式**（store 的字段为合成 `this$N`/`val$name` 且其值恰为对应参数/前序 load 的直传）重排为 super-先行；非合成模式（用户字段 pre-super 写）保持逐字并拒绝或如实呈现（那是 verifier 允许但罕见的人为字节码，不在源码形态内）。

## 处置

`recover-synthetic-ctor-super-order`（窄呈现切片）：ctor 呈现识别"前缀合成字段存 + super"序列（字段名匹配 `this$N`/`val$` 前缀或 InnerClasses/合成属性佐证，值为参数直传），把该组 store 移至 super 之后按序呈现；合成字段声明本身保留（faithful）。C1/C2 family 联编可过 javac 且行为一致。

原 class 为行为基准。


## 处置结果（2026-10-01）

已由 [recover-synthetic-ctor-super-order](../../../changes/recover-synthetic-ctor-super-order/) 闭合（root 验收 daa4fb31）：合成 pre-super 存组重排为 super-先行，C1/C2 family 联编可编译且行为一致；非合成/交错/计算值边界钉死。收尾由 root 在双 glm-5.3-flash 通道配额耗尽后代完成，记录见 [impl-record.md](impl-record.md)。
