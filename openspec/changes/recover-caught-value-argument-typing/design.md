## Context

[巡查证据](../../evidence/java-syntax-2026-10-01/catch-param-slot-reuse/README.md)：S1.twrHelper 的字节码里资源 `astore_0`（S1 型）与 catch 绑定 `astore_0`（行类型 IllegalStateException）先后写槽 0；handler 内 `tag(e)` 的实参读取值定义是绑定 store（Caught）。子句头拼写正确（行类型）；实参呈现类型取槽决策（S1）→ 转换拒绝。`Definition::Caught` 在 build.rs 已有类型事实（约 22953 行把 caught 值拼为子句类型），缺实参呈现路径的值归属。

## Goals / Non-Goals

**Goals:** 槽复用下 handler 体内参数读取按值定义（Caught → 行类型）呈现；S1.twrHelper 完整恢复 `return tag(local0);` 且整类可重编；无复用对照逐字不变。

**Non-Goals:** 槽决策机制重构（decide_types 单槽多定义的通用策略）；非 handler 场景的槽复用（finally/monitor 各自证书域）；行类型拼写本身（已正确）。

## Decisions

1. **值归属而非槽归属，落点最小。** 实参呈现解析读取局部时，若该次读取的 SSA 值定义是 catch 绑定 store（Caught）且存在以该行为子句的呈现上下文，类型取行类型（复用 22953 一带的 caught 拼写事实）；其余读取保持槽决策路径。若诊断发现实参呈现实际走"读取值→表达式渲染"路径（Caught 目前在 20119 报错、22953 有字符串），落点跟随事实，原则不变：呈现类型来自值定义的行类型。
2. **验收锚定**：S1 双方法（复用恢复、对照不变）、多 catch 各型子句的槽复用变体、17b 家族（T3/T1/C4/W17b）与 `p3_twr_enclosing_catch` 全绿；`p3_java_recovery` 既有期望不变。

## Risks / Trade-offs

- **值归属外溢到非 handler 读取** → 只对绑定 store 的 Caught 定义生效，且要求该行被呈现为子句；其余 Caught 读取（rethrow 呈现已有通道）不变，负例钉死。
- **行类型与实际运行类型不符** → 行类型即 verifier 保证的类型（catch 行命中的值必为其类型实例），呈现安全。
