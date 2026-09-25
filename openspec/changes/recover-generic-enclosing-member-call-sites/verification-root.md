# Root 独立验收（2026-09-25）

范围只包括已选 `Outer.A<T>` 静态成员作为非静态 `Plain` / `Generic<V>` 的封闭实例，以及四个独立调用方的构造点、方法头和 `A.mark` 限定符；不宣称完整嵌套类族源码可重编。实现复用 `member_inner` 的双向关系、构造前缀和同次 SSA/早空值检查；源类型路径仅由选定物理定义与擦除证明产生，没有另建通用 pass 或依赖 JADX 运行。

独立用最终 CLI（SHA-256 `ca04265a4f412d59c29d6bd4a26b7d9cb961f72ae13e77684831c0b9e57b5145`）运行 [`accept-jarde.py`](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/shape-matrix/accept-jarde.py)。`-g`、`-g:none` 各自将四份 Jarde caller 对原始 `Outer` 家族单独以 `javac --release 8` 重编，再替换调用方运行原 `Runner`，完整八行 stdout（含四次空值异常先后和 `mark` 效果计数）均与原 class 一致。四份 JADX 1.5.6 caller 对同一原始目标独立重编全部失败，均漏掉合格封闭实例；[矩阵分析](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/shape-matrix/analysis.md)保留具体诊断。Jarde 的 `class-source --evidence all/essential` 四份正文完全一致，每个 `make` 完整恢复、无 fallback，source map 含精确 BCI 集：Plain/PlainRaw 为 `{0,3,4,5,6,9,10,11,14,17}`，GenericObject/Typed 再含 20。多成员 `class-source --evidence-bci 0..2` 不存在单一 driver 范围，按现有 API 以退出码 4、`partial/jre_evidence_range_invalid` 明确停止；未把它误当作正文不变的证据裁剪。

负控核对：错泛型签名/擦除、双向成员行错形、缺失及歧义选定定义均不给目标证明；预算/取消直接停止。另以 `javac --release 8` 编译合法顶层名 `Outer$A` 及其成员 `Plain`，最终 CLI 保留物理 `$` 并在调用点留下 `@bytecode` 缺口，不把合法 `$` 标识符拆成未证明的嵌套边；这是当前证明范围内的保守拒绝。原 `SimpleOuter.Inner` 两条非泛型正例也在定向回归中重新通过。

根代理在最终代码上运行 `cargo test --lib --locked` 28/28、`cargo test --test class_source --locked` 47/47、`cargo test -p jarde-java --lib --locked` 180/180、`cargo fmt --all -- --check` 和 `openspec validate recover-generic-enclosing-member-call-sites --strict`，均通过。与 `recover-shadowed-method-type-parameters` 交叉验收：完整类 Java 8 重编、执行及泛型反射在两种 debug 设置下对齐；verifier-valid 错擦除 Signature 均被拒绝。严格 `cargo clippy -p jarde -p jarde-java --lib --locked -- -D warnings` 仍因共享树 17 项其它告警失败（[单独债务记录](../../evidence/java-syntax-2026-09-22/architecture-debt.md)）；本项新增的 `large_enum_variant` 与 `generic_return_candidate` 参数数告警已由窄改动消除，未借本项重构 enum switch/Region/其它候选链。此门禁未宣称通过。

实现代理私有 target 清理 3.7 GiB；Root 两轮独立测试 target 分别清理 3.7 GiB 与 3.0 GiB，仓库无常驻 Cargo target。仅保留 42 MiB 的最终验收 CLI `/tmp/jarde-generic-accepted-cli`，便于后续接口 `super` 变更作前置对照。未修改物理 classfile 来源或依赖版本。
