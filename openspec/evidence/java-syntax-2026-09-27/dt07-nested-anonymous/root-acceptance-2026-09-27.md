# DT-07 主线独立验收

固定 JADX 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。Luna 实现提交 `29c468f81b5989d010d148454908001038fe5fd7` 与已验收 DT-08 捕获实现合并后，root 从合并态构建 `jarde-cli`，SHA-256 为 `aa51d1b073a7b5758ae44abf61712d1b474aa0e717c29788e36cef43650e0ece`。独立执行 `dt07-nested-anonymous/replay.py fixed`，输出保存在 `/tmp/jarde-dt07-root-accept-merge/`；其摘要除 CLI SHA 外与实现者的固定重放逐字段一致。原源码、固定 JADX 与合并态 Jarde 的完整发行源码均由 `javac --release 8 -g:none` 重编，并经 `java -Xverify:all` 输出 `1`。`p/Action`、`p/Factory`、`p/Nested` 和两个物理匿名子类的 class-source 查询均成功；物理子类保持可独立查询，发行根源码只含嵌套 `new p.Factory() { ... new p.Action() { ... } }`。

根级复核了两条准确分配/构造 BCI、接口合同、类型化 `InnerClasses`/`EnclosingMethod`、内层唯一 immediate-parent 捕获字段及所选范围 XRef。嵌套候选在完整证明后通过现有方法和类源码 writer 放置；新增报告字段记录投影状态及表达式范围/物理锚点，拒绝时保留原物理文本。重复分配、外部引用、错误 enclosing/描述符、捕获读取、额外效果、预算和取消的测试均拒绝半份投影。与 DT-08 交界处保持该实现的精确捕获字段描述符，嵌套链的 owner XRef 则延后到两层关系闭合后清点。

合并态 `cargo test -p jarde --test class_source --locked` 为 82/82，通过 `cargo check --workspace --locked`、`cargo fmt --all -- --check`、`git diff --check` 与 `openspec validate inline-proved-nested-anonymous-interfaces --strict`。同一合并态 CLI 还独立重放 DT-08：原/JADX/Jarde 完整 Java 8 源码均重编、验证运行输出 `2.5\n-0.0`，摘要除 CLI SHA 外与 DT-08 原验收一致。当前证明只处理恰好两层的接口直接返回；匿名父类、捕获字段正文读取及更多层关系保留为后续独立任务。
