## Context

[巡查证据](../../evidence/java-syntax-2026-09-30/cf15-crossing-patrol/README.md)定位拒绝点为 `build.rs` 实参转换分派末端的 `no safe reference conversion evidence`。分派现有回答次序：lambda 工厂 → `java.lang.Object` 要求 → 同名 → 数组闭集 `array_reference_widens` → `reference_overload_calls` 证明 → `platform_reference_argument_widens`（仅 `java_release==8 && List→Iterable`）。C2.alias 的 `IllegalStateException → Throwable` 落空。上游 JDK 的 java.lang 异常层级是固定平台事实（与 List→Iterable 同一性质的闭集知识），JVM 校验器保证 catch 行命中的值必为该行类型的实例，上转型在字节码与源码两侧都无失败路径。

## Goals / Non-Goals

**Goals:** java.lang 异常子类 → java.lang 祖先（Throwable/Exception/RuntimeException）的**调用实参**上转型，按既有 widening 模式呈现（保留要求类型拼写，防重载重定向）；负例（非 java.lang、非祖先关系、数组/原始类型混入）保持拒绝。

**Non-Goals:** 用户类/接口层级（升级路径登记）；返回位置与非调用上下文；`Object` 目标（已有）；窄化（downcast 保持显式 cast 通道）；DEX。

## Decisions

1. **闭集表放 `platform_reference_argument_widens` 旁**，同风格函数（如 `java_lang_throwable_widens`）：表为 java.lang 核心 Throwable 子类到祖先的直接边（子类→父类链的传递闭包在函数内展开为查表可达）。表内容与 JDK 8 java.lang 固定层级一致，逐对在注释中注明来源；`java_release` 不限制（java.lang 异常层级在 8+ 稳定）。
2. **呈现走 `cast_argument` 保留要求类型**：与数组/List→Iterable 同款——源码写 `(Throwable) e` 形态的显式保持（javac 允许冗余上转型），防 overload 重定向；不为"更漂亮"的隐式上转型冒险。
3. **升级路径登记**：非 java.lang 呈现类型或要求类型出现时仍拒绝；未来以 resolution 环境的类层级查询建 `reference_overload_calls` 同款逐 BCI 证明，触发条件 = 首个真实用户类上转型案例（记录在案，不在本片实现）。

## Risks / Trade-offs

- **表错对（非真祖先）→ 错误隐式转换** → 只收录 JDK 固定层级直接边，逐对可对 javadoc 核对；负例含表外类型（如用户类 `MyException → Throwable` 拒绝，直到升级路径启用）。
- **重载重定向** → 呈现保留要求类型拼写（决策 2），与既有 widening 分支同一防护。
- **作用面外溢到返回位置** → 判别只挂在实参分派（该函数本身只服务调用实参），返回位置走各自通道不变。
