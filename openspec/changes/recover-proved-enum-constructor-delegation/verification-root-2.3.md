# Root 独立验收：2.3 双构造器整组投影

Root 从最终工作树重新运行定向枚举测试 **24/24**、`class_source` 集成测试 **47/47**、`anonymous_allocation_candidates` **4/4**，并重建 `jarde-cli`；CLI SHA-256 为 `2f094b8f16a8152d828bde534aa3719eb64cdf2626b63f62448686a2456b3c8c`。`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-enum-constructor-delegation --strict` 通过；Clippy 退出 0，仍有 2.2 已记录的 10 条告警，没有新告警。

Root 在独立 `/tmp/jarde-enum-root-2-3.xUPr1f` 中，从冻结 Java 源码分别以 `javac --release 8 -g`、`-g:none` 构建原 class，运行 JADX 1.5.6 与当前 Jarde CLI，并将各自的完整源码作为 Java 8 重编。公有枚举的 Jarde 源文件使用合法文件名 `DelegatingEnum.java`，未改生成文本。三份 class 均通过 `java -Xverify:all`，两种模式逐字输出 `values=ZERO:0,ONE:1`、`effects=2:0,1`、`declared-constructors=2,3`；Jarde 两份枚举源码 SHA-256 均为 `c748f28b586e93a7763f83314a7ba9db7e21cf294d4846283de1fa1a4f68ded5`。这同时排除将 `ZERO` 偷换成 `ZERO(0)` 后丢失无参重载的表面等值输出。

Root 从仓库外复制冻结的 `negative-controls/replay.py` 与源码，仅把副本中的预期 CLI SHA 换成当前值；脚本重新编译、验证、执行原/JADX 变体并检查 Jarde，**9/9 verifier-valid 控制通过且 Jarde 均拒绝整组投影**。当前 CLI 的默认/all 证据对 `-g`、`-g:none` 输出相同正文；公开 JSON 保留两条物理构造器 `(Ljava/lang/String;I)V`、`(Ljava/lang/String;II)V`，均为 `recovered/structured`。单构造器 `Stage`、`Measure` 的默认/all 正文仍分别等于此前冻结 SHA-256 `edc4e0bf9d78654f3b16247a709d43be717261edeaaf4362c81f80c583a5d5d7`、`a639494bc370c73056aa70f5da4e210d88ab587f181a8cab7ea4ffb1b8bd42c9`。

代码审查确认双构造器先走原有完整 `values`/`valueOf`/`$values`、全方法使用 census 和 `<clinit>` Code/AST 门，再发布 `ProvedEnumConstantGroup`；两个构造调用按 BCI 与各自描述符配对。终端用户语句通过既有 Java AST emitter 写出，已证明的物理 slot 3 映射为源码 `arg0`、slot 0 接收者映射为 `this`；失败时不发布部分投影。2.4 的来源/预算/取消与公开 JSON 全量核对、3.1 最终整案验收仍待进行。本轮私有 Cargo target 留给 2.4 复用，验收后清理。
