# 修复后整类对照

本目录用重建后的完整 CLI 重放 Java 8 静态限定符 fixture、1.2 边界类及 owner mismatch 补丁。先在仓库根目录构建 CLI，再运行：

```text
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo build -p jarde-cli --bin jarde-cli
python3 openspec/evidence/java-syntax-2026-09-22/static-field-qualifier/post-fix/run_post_fix.py
```

脚本固定主 fixture 的 class hash，并对源文件、JADX、Jarde 三个完整类分别 `javac --release 8 -g:none` 后以 `java -Xverify:all` 执行。它比较静态字段读取、简单写入、复合写入、生产者/RHS 异常顺序，以及边界类里的限定调用、普通弃值调用、接口静态方法和继承静态方法。所有命令输出、完整生成源码、版本和 hash 记录在本目录；`summary.json` 中四个可执行完整类的三方结果均逐行相同。

另一个已通过 JVM `-Xverify:all` 的 CP-only `Child.ping` → `Other.ping` 补丁保留为失败边界：修复后的 Jarde 生成带 bytecode 来源的拒绝文本，不再输出 `.ping()` 错误改绑表达式。拒绝正文不是完整可编译 Java 方法；这是有来源的保守 fallback，source-map 回归另在 `tests/p3_popped_static_qualifier.rs` 校验 BCI 0、1、2、5。

本项未修改 census/fingerprint。`p3_type_qualifier::static_owners_survive_generated_parameter_and_suffix_names` 是已有独立 RED；冻结修前 CLI 也复现相同输出，归属 `preserve-type-qualifier-bindings`。
