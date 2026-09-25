## Why

普通类的 blank `static final` 字段目前在已恢复的初始化块中写成 `Class.FIELD = value`，实际 javac 拒绝该赋值；顺序、分支与局部变量样例均可重复。直接删除限定符又会让恢复局部名遮蔽字段，必须同时闭合字段身份与名称绑定。

## What Changes

- 在当前普通类的已知 `<clinit>` 内，对同源字段声明证明的 blank static final 写入使用简单字段名，保留原赋值位置与求值次序。
- 让既有字段赋值 AST 表达无接收者的简单字段写入，并在既有名称分配中避开这些必须裸写的字段名；不以文本替换或伪造局部赋值完成。
- 固定顺序、两分支、调用次数、`local0`/后缀冲突、来源与预算回归，实际重编译完整恢复类并对照原 class 和 jadx。
- 前置为已有 field、declaration、names 与 static-initializer completion 路径；接口字段声明初始化、枚举整体恢复、非法 final 写入修复和任意初始化表达式上提均为非目标。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 在已有区域内保留普通类 blank static final 字段赋值的合法简单名及其绑定，不改变效果和来源。

## Impact

预期涉及 `jarde-jvm::MethodIr` 既有类头借用视图、`jarde-java` 的 report/names/build/ast/emit 与集成测试。复用当前 ClassFacts 和 MemberHeader，不读取其它类、不增加 resolver、pass、缓存或依赖；CLI、宿主协议以及验证/编译结论契约不变。接口样例的独立失败单列证据，不并入本项。
