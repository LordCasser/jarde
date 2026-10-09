# 构造引用数组元素 fixture v1

此 fixture 为 `compose-constructed-reference-array-elements` 的独立双 javac 正例。完整 source family 包含七个同包 top-level 类型：`Main`、`Base`、`Mid`、`DirectA`、`DirectB`、`TwoHop`、`LocalInterface`。六个目标方法分别覆盖平台 `CharSequence[]`、`Collection[]`、`Throwable[]`，以及自有直接子类、两跳子类和接口实现数组；每个数组有两个 inline `new` 元素。构造参数通过 `mark` 逐项打印，原有构造器也打印类和参数标签，main 的确定性双元素 observer 打印运行时类名。

冻结的源计划为 [source-plan-v1.md](source-plan-v1.md)，SHA-256 `0c5929572e00b7924effe74203cc6017bc9c7234f98223653d048da3e9631f02`。独立构建脚本为 [build_fixture_v1.py](build_fixture_v1.py)，最终运行脚本 SHA-256 `fff9bddbe9c51d11165ea9265af5a9bd2f6e947202fb431543433ecfcc134963`。最终原始构建记录是 [fixture-v1/run-003/manifest.json](../../../openspec/changes/compose-constructed-reference-array-elements/results/fixture-v1/run-003/manifest.json)，SHA-256 `98e8f014b4cb7f8a4efca50ff54e7fee992805f27387f95d4c76db1f9b5bc713`。它记录实际命令、Corretto 8/OpenJDK 23 工具哈希、七个源和 class 哈希、逐腿 javap、退出码及 stdout/stderr 原始文件和摘要。

两条腿均使用显式空 classpath/sourcepath，完整编译全部七个类并以 `-Xverify:all` 运行。stdout 完全相同，SHA-256 `7f3031d85412a384e192dfe07b654d0aab6cfcd3dac85be8b795358e7f5651ee`；stderr 均为空，SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`。两腿所有目标方法都记录了两次 `new` 和 `invokespecial`。平台 Collection 的 constructor 参数另包含两个 `String[]` 与 `Arrays.asList`，所以该方法还保留了额外 `dup`/`aastore` BCI；这些真实指令已完整保存在 manifest 的 `target_method_opcodes_bci`，没有从清单中剔除。

`run-001` 与 `run-002` 是相同冻结源的历史构建记录；`run-001` 的首版检查器对 Collection 参数内部的数组 store 套用了错误的统一计数，故其 opcode-shape 检查为 false，真实编译和运行仍成功。`run-002` 记录了分方法计数；`run-003` 保留相同源码和命令并固定了最终 manifest 检查名称与预期计数。对应 runner 原始 stdout/stderr 也保留在本结果目录。Candidate 重放与产品验收尚未包含在本 fixture 构建记录中。

**勘误：** `source-plan-v1.md` 中“同一十源文件集”是规划文字里的旧计数笔误。实际源码闭集为 7 个源文件、7 个 top-level class；run-003 JSON 的 `fixture_sources`、两腿各自的 `class_files` 和原始 javac argv 均对应这 7 项，且两腿完整编译运行。冻结的源计划和历史 manifest 均保持原样。为供 integration `include_bytes!` 与 reader census 使用，`javac8/classes`、`javac23/classes` 是从 run-003 对应的 `original-classes` 逐字节复制的 14 个 class；复制来源与 SHA-256 由 `fixture-class-copy-v1.json` 记录，没有重编。

原始 stdout/stderr 也复制到 `oracle/javac8`、`oracle/javac23`，用于新 integration 的逐字节 oracle；`runtime-oracle-copy-v1.json` 将四个副本与 run-003 命令输出的 hash 一一核对。两份复制记录都保留 run-003 manifest 的原 hash。

完整 integration 位于 `tests/p3_constructed_reference_array_elements.rs`。它针对两腿各自构建同一 snapshot 的七份 class-source，再仅用七份生成源隔离编译；冻结 helper 源只用于源/原class参考，不会补入候选编译目录。该宿主JDK测试不替代 root 的双原JDK CLI 重放。fixture 与结果文件的最终 hash inventory 见 `results/fixture-v1/file-hashes-v2.json`（inventory 文件不把自身纳入哈希）。
