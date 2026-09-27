## Context

现有 `src/facade.rs::project_class_source_anonymous_super` 已证明 Java 8 child、准确 `InnerClasses`/`EnclosingMethod`、唯一分配点、owner 全局引用、准确父类构造器及所有 child 方法完整性，然后调用 `crates/jarde-java` 的同次 AST emitter。它目前要求 child 构造器具有 `GenericConstructorCandidate`，该证书只表示 `super(...); return;` 的无效果或直接参数转发。DT-09 隔离样例的物理构造器是 `aload_0; invokespecial Base.<init>; iconst_1; putstatic Subject.value; return`，同次恢复已有正确语句，但不能取得无效果证书。

## Goals / Non-Goals

**Goals:** 沿现有类级投影路径加入一个有界的带初始化效果构造形态；只在整棵根类源码可原子呈现时隐藏源级匿名 child。

**Non-Goals:** 不扩大到 JDK `Thread`、非返回表达式、外层私有实例字段、捕获参数、多个初始化语句或嵌套匿名图；这些与实例初始化块本身有不同的证明前提。

## Decisions

1. **给带效果构造器单独的同次证书，而不复用无效果泛型构造证书。** 在现有 `jarde-java` class-source sidecar 内检查完整 Code、单块 SSA、无 handler、准确 AST `super(); <字段写>; return;`、各指令与操作及效果关系。只接受一个可拼写的根类静态 `int` 字段赋值；常量、字段 owner/descriptor 与 AST 值必须一致。可以用私有小结构或函数传递已证明的初始化语句，避免新增公开模型、crate 或泛化框架。备选是把 `GenericConstructorCandidate` 放宽为任意后续效果，这会破坏其“参数未用/准确转发”语义，故不采用。
2. **沿用现有匿名体 emitter 和根类原子提交。** 在 child 所有方法与构造器证书全部通过后，仅从同次 AST 呈现已证明的初始化语句，先放 `{ ... }`，再放覆盖方法；与唯一根方法替换一起提交。不能从已经渲染的构造器文本剪切 `super()` 或 `return`。物理 child 报告仍保留。
3. **保持解析、选择、验证、源码恢复分层。** 读者仍只给物理事实；运行环境与类级关系仍由 facade 选择；Jarde Java 层证明构造语句并呈现源码。不会执行目标代码或增加依赖读取。现有输出/分析预算与取消语义须覆盖新证明及呈现工作。

## Risks / Trade-offs

- 单个静态赋值只是原 JADX 测试的独立子形态，不能以此宣称 `Thread`、私有外层字段或 `.start()` 分配位置追平；账本保留剩余边界。
- 构造器 AST 可能包含由编译器插入的非源语句；以完整指令/SSA 对照与唯一形态门槛拒绝，避免改变效果顺序。
- 字段赋值可能触发初始化或异常；只重排为源级 `super()` 后的实例初始化块，不改变效果相对位置，并用完整源码重编验证运行测试。
