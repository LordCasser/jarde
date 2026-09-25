# 类级声明注解验收

原始 Java 8 完整类与 JADX/Jarde 重编译类的三行反射输出均为 `true`、`3`、`@java.lang.annotation.Retention(RUNTIME)`。另一个 CLASS-retention 完整类三边均为 `0`、`null`；原始与修后 class 的 `javap -v -c -p` 都有 `RuntimeInvisibleAnnotations`、`LHiddenTag;` 与 `value=5`，没有把不可见注解当作运行时可见。

root 从最终源码以 `CARGO_INCREMENTAL=0 cargo build -p jarde-cli` 重建并冻结 `/tmp/jarde-cli-class-ann-root-final`，SHA-256 为 `89b0d40ba45f69034d8e0ba0e9e22cfbea08655381ecd743b1f96e379f28d59a`。在独立复制的 `/tmp/jarde-class-ann-root-I3hs1Y` 中，分别重放 `class-annotation-uses/boundaries/run_boundary_audit.py` 和 `implementation/run_implementation_audit.py`，输入源码/class 哈希与代理证据相同。重复属性条目保留原壳和两条拒绝，非法 Java 类型名保留原壳并拒绝，损坏 value tag 保留原壳且以 `classfile_invalid_attribute_content` 停止；文本与 JSON 均未输出部分注解。

结构审查确认：`ClassMemberFacts` 只在成员表走完后读取类属性壳，按物理顺序对每个壳计 `AttributeBytes` 和 `ResultItems`；有成员前缀停止时尾部不可达，壳列表为空。`Engine::class_source` 仅在存在类注解壳时读取其内容；有方法体时复用已准备的常量池，无方法体时惰性读取一次。reader 的同一 `element_value` 读树与 `MemberDefault` 受限拼写承载 marker、枚举、具名数组及嵌套值；重复类型和不可拼写值逐条拒绝，字段/方法/参数与 type-use 属性不被移动到类头。

root 门禁：`class_source` 27/27、`p3_annotation_default` 8/8、`class_annotation_facts` 7/7、reader lib 160/160、CLI `class_source_cli` 16/16、`historical_classfile_corpus` 1/1、`p5_corpus_fingerprint` 5 通过/1 ignored；`cargo fmt --check`、`git diff --check`、47 个 OpenSpec change strict 均通过。reader census 已重测为 141 类、1049 个 Code；新增 24 个注解夹具按原始文件哈希加入 corpus fingerprint。reader library 严格 Clippy 及根库/注解定向目标在排除既存 `region.rs` 的 `type_complexity` 后通过。

`p5_bulk_corpus` 仍有三项既存 RED：flat-mixed 与 direct arm 的计数 pin 漂移、`Guarded.boom` 分类迁移。它们在 `../../evidence/java-syntax-2026-09-22/bulk-recovery-pin-drift/analysis.md` 独立记录；本项不重新钉住批量计费表，也不宣称全仓门禁全绿。
