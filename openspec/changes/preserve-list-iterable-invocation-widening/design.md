## Context

见 [proposal.md](proposal.md) 和 [独立调用探针](../../evidence/java-syntax-2026-09-27/cf10-foreach/isolation/list-iterable-call/)。共享调用实参路径已有 descriptor 参数计数、目标类型拼写和 Cast 来源；引用参数目前只接受同型、Object、数组或本 run 已证明的重载转换。`ListToIterable.main` 的实参是 `Arrays.asList` 调用，呈现为 `java.util.List`，物理目标形参是 `java.lang.Iterable`。

## Goals / Non-Goals

**Goals:** 复用调用参数表达式与来源结构，为 Java 8 平台中的精确 `java.util.List` 到 `java.lang.Iterable` 上溯增加可审计的安全证据，使该表达式可交给既有目标参数类型路径。

**Non-Goals:** 通用类/接口层级解析、其它平台类型对的白名单、用户自定义继承关系、泛型参数可赋值性、赋值/返回位置转换、重载解析新机制，以及步长索引循环的 Region/局部生存期恢复。

## Decisions

1. **在现有调用参数证据边界扩展一个精确平台关系。** 只对完整呈现名 `java.util.List` 与目标 `java.lang.Iterable`、Java 8 RuntimeProfile 准入；复用调用实参现有显式目标类型表达及 BCI 来源。考虑过直接接受任意 Java 可转换引用，但当前层没有一般层级关系读取，且会让 descriptor 被误用为安全 cast 证明。
2. **只使用已知 Java 8 语言/平台契约。** 该关系来自标准 `List` 继承 `Collection`、再继承 `Iterable` 的平台类型合同，不读取宿主 classpath、不执行目标代码、不下载依赖。考虑过在单类策略下加载平台 class 文件；这既超出调用点需要，也会引入宿主环境依赖。
3. **不改调用目标或求值计划。** 目标仍取物理调用 descriptor；添加的类型呈现包住原实参表达式一次，并将转换与原实参 producer 和 invoke BCI 关联。重载歧义与其它引用关系沿原有证据/拒绝路径。

## Risks / Trade-offs

- [呈现类型使用短名或泛型形态导致误准入] → 只允许完整 `java.util.List`，不从 `List<T>` 参数化细节推导兼容性，也不以短名匹配。
- [扩展被误读为通用平台子类型支持] → 在代码、拒绝诊断和验证报告中把准入限定为单一类型对，并以未证明引用关系负例检查旧拒绝仍成立。
- [新 cast 改变求值次数或运行行为] → 只包装现有表达式，不复制或移动实参；完整类重编与运行输出以及来源 BCI 一并验收。
