## Context

[巡查证据](../../evidence/java-syntax-2026-10-01/inner-enum-args-patrol/README.md)（预研段）固定了字节码结构；`ClassSourceNestedEnumFamily`/`MemberFamily` 通道已能准备子类报告（`child: Box<ClassSourceReport>`，逐方法恢复文本在内）。第一个取证义务：确认该通道今天是否会为**匿名**枚举子类（`N2$Operation$1`，InnerClasses 匿名标记）准备报告——若只准备具名成员，本片需把匿名子类纳入准备集（仍是既有通道的输入集扩展）。枚举自身 ctor 在此形态带合成 `$1` 末参（防递归），常量步骤的 invokespecial 指向 Sub ctor。

## Goals / Non-Goals

**Goals:** N2 全量折叠（常量体含 @Override 方法文本）；family 输出消除独立匿名子类呈现；三方重编运行一致；无匿名体路径零变化。**Non-Goals:** 常量体带字段/构造（`PLUS(1) { … }` 混合形态——构造实参与匿名体叠加，随后按需另片）；子类体方法非 structured 的折叠（保持逐字段）；接口 default 方法呈现；DT-12 其它匿名形态。

## Decisions

1. **子类义务证明放在常量折叠侧**：常量步骤的 `new Sub` 目标查已准备子类报告集；义务 = 关系（InnerClasses 成员 + extends 本枚举）+ ctor 委托体逐指令（含合成 `$1` null 参）+ 成员集恰为覆盖方法。任一不满足 → 该常量按逐字段（整组保守，沿用现有组级原子性）。
2. **呈现复用子类报告的方法文本**：常量体 = 子类报告的实例方法恢复文本按声明序拼接（@Override 标注按既有 direct-override 通道判定——同 snapshot 接口方法表已有先例）；子类独立呈现从 family 输出中抑制（它已被常量体消费）。
3. **匿名子类准备集**：若现通道不收匿名类，扩展准备输入集为"枚举的 InnerClasses 成员中 extends 本枚举的类（含匿名）"，沿用既有准备预算与拒绝语义。
4. **验收锚定**：N2 家族（Operation + IOperation + 匿名类）三方 `javac --release 8` 重编、`java -Xverify:all` 运行 `7`/`7` 一致；`Operation$1` 不再独立呈现；N0/N1/N3 与四固定形逐字不变。

## Risks / Trade-offs

- **匿名子类准备引入额外类读取** → 走既有家族准备预算（A16 家族装配已有计费先例），不加新预算维度；拒绝语义保持。
- **子类体方法恢复质量** → 仅 structured 方法参与（MVP 边界）；拒绝时整组逐字段（保守），负例钉死。
- **@Override 判定跨类** → 复用 direct-override 通道的接口方法表事实；判定不了就不加标注（呈现正确性优先）。
