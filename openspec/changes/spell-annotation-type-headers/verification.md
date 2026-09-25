# 验收记录

root 审阅 `class_declaration` 的局部分支：仅在 `ACC_ANNOTATION | ACC_INTERFACE` 同时存在且物理接口表**恰好一个** `java/lang/annotation/Annotation` 时，将隐式接口从 Java 头部省略。`ClassSourceDeclaration.item` 的 flags 与接口表原样保留；普通接口继承及非 canonical 注解接口表继续经原发射路径，不新增 reader/AST/pass/公开字段。

最终 CLI SHA-256 为 `7a33dbed5b390009cb65802fd9264367e1d5854b9cc70dbced5ae1878109c822`。root 在临时复制的固定 Java 8 fixture 上重新编译原 class 并比对 SHA，独立重放原/JADX/Jarde 的完整类与未改反射 runner；证据在 `../../evidence/java-syntax-2026-09-22/annotation-default-boundaries/header-minimal/post-fix-root-replay/`。Basic 三方均 javac 成功、`java -Xverify:all` 输出同为 `5`、`source-only`、`[2, 4]`。Nested 两类头已合法且 Jarde 全类可编译，但缺失嵌套默认值使反射运行失败，保持下一 change 的独立 RED。

门禁：`p3_annotation_default` 5/5、`class_source` 16/16、复合赋值 4/4、`p5_corpus_fingerprint` 5 通过/1 ignored，reader 160/160；`cargo fmt --all -- --check`、`git diff --check`、`openspec validate --all --strict` 62/62 均通过。`cargo clippy -p jarde-java -p jarde-reader --all-targets --locked -- -D warnings -A clippy::type_complexity` 通过，唯一排除的警告是既存 `region.rs:1736`。`p5_bulk_corpus` 的 3 个独立 RED 仍按 `../../evidence/java-syntax-2026-09-22/bulk-recovery-pin-drift/analysis.md` 处理，不属于本头部修复。
