## Context

[巡查证据](../../evidence/java-syntax-2026-10-03/member-class-folding-patrol/README.md)：M1（Base/Ctrl/Deep/Err/Inner 五直接静态子）/M2（Solo 单子）fam/solo jar。既有机械（先例）：`src/member_inner.rs::FamilyRootScan`（InnerClasses 直接行扫描——现 one-child + DeclarationPair 枚举投影对）、枚举折叠投影（`enum_constants.rs`/facade——名字重写 token 机械、nn 片刚接入其拼写重写）、成员读取与 A16 计费。**第一个取证义务**：读 `FamilyRootScan` 的多子拒绝位与 facade 的投影装配消费（`prepare_class_source_member_family`/`project_class_source_member_family`），确定 (a) 多子化的最小改动面、 枚举折叠投影的名字重写机械可否直接复用于"成员文本嵌入+引用重写"、 孙代与 InnerClasses 嵌套行的处理顺序。

## Goals / Non-Goals

**Goals:** 任意数量直接静态成员类/接口折叠（一层）；M1/M2 家族重编行为一致；分离呈现/匿名/枚举投影零回退。**Non-Goals:** 非静态成员类（this$0——后续片）；孙代折叠（子类文本内池拼写，登记）；局部类；import/包结构（既有全限定惯例）；`Signature` 泛型投影（既有边界）。

## Decisions

1. **多子化扫描**：`FamilyRootScan` 静态行收集为列表（保持逐行判据：源名可拼、flags 源码可写、可见性单一），不再 one-child 拒绝；DeclarationPair（枚举投影对）路径优先不动。
2. **折叠投影**：外围文本尾部按行序嵌入子类文本（`static`/flags 按行 access_flags 拼写）；名字重写机械从枚举折叠投影提取复用（`M1$Deep`→`Deep` 于折叠作用域内全部引用位——类头/字段/throws/体内/子类间）；单类输入（无子类定义）保持现分离行为。
3. **验收锚定**：M1 fam.jar（`hi`/`ok`）、M2 solo.jar（`7`）整 jar 重编一致 + 变体（接口型子、子类 extends 兄弟、跨类引用者 M1User 分离呈现不变）；负例（孙代 `Inner$Leaf` 不折叠、非静态子保持分离）。

## Risks / Trade-offs

- **名字重写漏位/误重写** → 作用域=折叠投影文本（枚举先例的 token 边界机械）；分离呈现 diff 钉死。
- **子类文本含拒绝方法** → 折叠嵌入保留子类自身引注语义（不因折叠隐藏）；MVP 接受带引注子类折叠，行为差测试覆盖 M1/M2 全恢复路径。
- **计费膨胀** → 折叠=每子一次既有恢复，按成员计费沿用；预算/取消原子性测试。
