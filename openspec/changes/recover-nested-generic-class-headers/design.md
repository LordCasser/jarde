## Context

[巡查证据](../../evidence/java-syntax-2026-10-03/nested-generic-header-patrol/README.md)：`Z1$Box` 三重阻断。既有程序：`signature.rs` 头投影（顶层已证）+ `jvm_signature_scope_unproved` 的 scope 机制（655/1143 行附近）+ 折叠片逐行判据（InnerClasses 行源名/kind/名字）。**第一个取证义务**：读 `class_generic_source_unproved` 产生点（signature.rs 或消费层）——具体哪几位（kind/nesting/name/type-use annotations）未证、顶层路径证明了哪些位；确定嵌套位的证明数据源（InnerClasses 行 vs 类自身 header）与 scope 注入点（成员投影取 scope 的路径）。

## Goals / Non-Goals

**Goals:** 嵌套泛型类头投影 + U 域链解锁；Z1/Box 呈现 `Box<U>`/`U value`。**Non-Goals:** 同类绑定链（generic_call_binding_unproved——下一片）；type-use 注解头位（若为独立未证位，登记）；通配符投影细节（既有边界）；折叠防线变更（语义不变，按评估如实报告）。

## Decisions

1. **嵌套位证明复用折叠判据**：InnerClasses 行（源名可拼、kind=member、名字=Outer$Simple 拼接一致）+ 类 header kind 与 access_flags 对应——与折叠片同一判据源，避免第二套。
2. **scope 注入**：头投影成功即向该类成员投影提供自身类型参数域（既有 scope 机制的输入位）；失败保持现拒绝链。
3. **验收锚定**：Z1 分离与 jar 两口径（Box 头 `Box<U>`、`U value`；Z1 自身 `List<T>` 字段仍按现状〔同类链〕如实呈现）；变体（非静态嵌套泛型、双参 `<U,V>`、bound 形 `<U extends Number>`）；负例（行不可拼/名字不一致保持三重阻断链原样）。

## Risks / Trade-offs

- **头投影与折叠呈现序**（折叠文本内头拼写）→ 头投影产出 Signature 文本片段，折叠嵌入位沿用（两者正交：先投影后折叠或折叠时请求投影——按取证选最小面）。
- **type-use 注解位独立未证** → 如实登记，头投影按已证位呈现。
