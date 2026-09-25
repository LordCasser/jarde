## Why

自写 CastAudit 的静态字段初始化被恢复成 `static { value = "ok"; return; }`，实际 javac 拒绝。`<clinit>` 已由现有声明事实识别，但正文 formatter 仍按普通方法写尾部返回；缺口位于呈现边界。

## What Changes

- 将已识别静态初始化块最外层最后一条无值 return 呈现为块的正常结束，保留终止 BCI 对实际闭合大括号的来源映射。
- 两遍 emitter 使用相同决定，继续执行输出预算、来源预算与逐字 replay 校验。
- 普通方法、构造器及非尾部返回保持各自语义；不以删行方式消去提前退出或其后效果。
- 用原 class 与实际恢复整类的重编译/执行验证初始化值、效果次序和初始化失败。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：约束静态初始化块的尾部正常结束及其来源呈现。

## Impact

前置为现有 DeclarationForm::StaticInitializer、Return AST、统一 emitter、source-map replay 和 class-source 包装。主要修改 `jarde-java/src/emit.rs`，配套定向 fixture/test；无需新 AST 节点、区域规则、pass、状态枚举或依赖。

非目标：任意 `<clinit>` 的提前退出结构化、异常/循环区域扩展、裸 throw、字段初值上提、枚举声明重建、声明/验证平面重设计。空初始化块仍按实际发射内容分类，不为保住统计数添加无意义语句。已有其他不支持的正文不因本项被声称完整恢复。
