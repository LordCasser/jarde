## Context

固定输入和旧组合证据见 `openspec/evidence/java-syntax-2026-09-27/dt29-reference-cast-audit/combined/`。旧 `results.json` 是先前基线，不是当前主线状态。已完成 A 的 accessor、B 的 public/private 写入及同包 protected/包可见写入；余下 C/D 的接收者是 B，而 setter 声明类是 C/D。`D.set` 的物理 `(B,Z)V` 与泛型 `<T:B>(T,Z)V` 一致。根类 `run` 的拼接已获证，但 `bits(A)` 的私有调用转换缺证；`bits` 四组 `ifeq`/φ/`append` 分布在多个块，现有同块 concat 证书明确拒绝 split。

## Goals / Non-Goals

**Goals:** 三个互不依赖的机制包各有完整类族 Java 8 正负例，最后固定九个物理类一次整体重编、验证运行；所有新证书以实际 BCI、物理 CP、选中定义及 SSA 值为依据，保持来源、预算、取消和安全拒绝。

**Non-Goals:** 一般 Java 类型系统、任意继承深度/跨包 protected、任意泛型正文、任意跨块拼接、CF-08/CF-16 区域债务，或逐字复制 JADX 格式。JADX 输出是对照与算法参考，语义和物理 class 是判定依据。

## Decisions

1. **P1：将声明类与实际接收者分开证明。** 在既有选中类池和父字段/引用实参证书上逐 BCI 核对 C/D `putfield` 的 A owner、唯一字段与 B 接收者，再核对 A.access$002 的精确私有目标和根类三处 `bits(A)` 私有调用。新证书仅在调用点已有的物理定义和源级重载唯一性均闭合时消费；不把“能 cast 为 A”当作所有同名成员都安全的全局规则。JADX `ModVisitor.fixFieldUsage` 的源级 cast 与 `ShadowFieldVisitor` 的遮蔽处理可参考，但固定 class 无物理 `checkcast`；保留 Jarde 现有准确 owner cast 和物理 accessor 形式。比在 printer 上统一强制 cast 更早阻止误绑。
2. **P2：泛型投影保持物理正文不变。** 复用 reader 的 Signature 解析和擦除核验，在 `class_source` 对完整直线 void 正文加入有界参数使用检查，确认被投影为 T 的形参槽未被写入，正文完整且所有参数引用来自同一 AST/SSA run；AST/SSA 仍用物理 B。固定 D 的四次 P1 字段写入及精确 accessor 调用可在源码里保留为 B 类型操作，因为唯一 class bound 擦除为 B，P1 将 A 字段访问和 accessor 目标显式钉定，且正文不把 B 值写回未知子类型 T。`$` bound 名称只可在当前选中 class 自己的 `InnerClasses` 记录证明它与 bound 名称具有同一 outer 和合法直系 inner 名、bound 与已验证物理参数擦除完全相同、同一 physical descriptor 已有该类型的源码拼写时复用；此表证明的是 classfile 声明的 nesting 关系，不声称 B 的定义也已被独立解析。不做字符串 `$` 到 `.` 替换，也不放宽一般 generic-name helper。预算/取消发生在发布泛型声明前；独立完整 Java 8 类族先证明反射/擦除和负例，再以 P1 固定 D 四写入正文做集成验证。
3. **P3：跨块拼接先证明整条链，再原子投影。** 复用已有 concat 身份、转换规则与 builder 语句能力，有限遍历四组条件与 φ、同一个 StringBuilder 的追加链和最终返回；要求唯一 builder、无别名、准确 append(String) 次序、每个字段 getter/私有 getter 一次、无未计入效果/异常边。能在现有 builder 中顺序输出等价语句则无需新增 AST 机制；否则仅增最小可证明的条件值表示，不放宽 `jre_concat_split` 的入口条件。参考 JADX `SimplifyVisitor` 的链识别，但不照搬无验证的块折叠。
4. **集成点保持三包解耦。** P1 核对成员与调用目标，P2 核对泛型源级声明，P3 核对值流与求值位置；三者共同使用已选物理定义、BCI 与 SSA，而非各自引入类层级推断器。P1/P2/P3 可在隔离 worktree 并行，固定 D 正例依赖 P1+P2，固定根类依赖 P1+P3；合入后一次重跑整体门槛。

## Risks / Trade-offs

- **误绑隐藏字段或私有重载** → 用反射同时读 A/B 同名字段、加入 `bits(B)` 竞争负例，并检查物理 CP 身份及逐点来源。
- **泛型表面正确而擦除错误** → 比较反射 `TypeVariable`、上界、泛型参数及普通参数类型；错上界/错擦除/正文不兼容均拒绝。
- **跨块拼接改变调用次数或条件顺序** → 将四次字段读取、φ 消费与 append 链作为同一证书；加入别名、复用、额外效果/异常边负例，未闭合不发布部分源码。
- **并行实现碰撞** → P1 以 facade/field 证书为主，P2 限 class_source 泛型头，P3 限 concat/build；共同文件改动在 root 合入时逐项审阅，不让单包自行放宽另一个包的门槛。
