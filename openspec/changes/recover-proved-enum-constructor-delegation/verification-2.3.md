# 2.3 双构造器整组投影验证

2.3 已将双构造器证据接入原有 `prove_group` 完整门。双构造器分支先证明 2.1 委托边和 2.2 终端正文，再继续走既有 `$VALUES`、`values`、`valueOf`、`$values`、物理成员使用 census、`<clinit>` Code 前缀及同轮 AST 检查。构造器调用按常量调用点 BCI 与各自 descriptor 配对。只有整组门全部闭合，私有 `ProvedEnumConstantGroup` 才携带无参构造器索引、终端构造器索引和已证终端 AST；拒绝分支不再保留 2.2 过渡用的正文候选。

类源码一次构建完整投影：常量按 `ZERO, ONE(1)` 发射，无参物理构造器在其原方法表位置写 `this(0)`，整数构造器在原位置保留 helper 调用和字段保存。终端正文取自同轮结构化 AST，经已有 Java 语句 emitter 输出；Code/AST BCI 已证明 slot 3 的两次读取后，发射边界把 helper 参数及字段 RHS 的物理局部名映射为源参数 `arg0`，并把 BCI 10 的 slot-0 接收者映射为 `this`。发射正文省略已证明的 `Enum.<init>` 前缀和编译器终结 `return`。两条物理构造器及其报告项仍保留，投影文本不出现注入的 name/ordinal。

验证结果：

- `cargo test --target-dir /tmp/jarde-enum-delegate-projection-target -p jarde --lib enum_constants::tests -- --nocapture`：24/24 通过。覆盖完整组正例、委托与终端边错、错槽/错误 helper/字段、Signature 缺失或错误、额外语句/效果、handler、预算与取消拒绝，以及输出预算不足时两条物理构造器仍整组保留。
- `cargo test --target-dir /tmp/jarde-enum-delegate-projection-target -p jarde --test class_source -- --nocapture`：47/47 通过；`class_source::tests`：20/20 通过。
- `cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-enum-constructor-delegation --strict`：通过。
- `cargo clippy --target-dir /tmp/jarde-enum-delegate-projection-target -p jarde --lib --no-deps`：退出 0，10 条告警与 2.2 根验收记录的既有告警数相同，没有新增告警。

运行时对照使用冻结证据目录的源码、helper 和 runner。最终私有 CLI 由 `cargo build --target-dir /tmp/jarde-enum-delegate-projection-target -p jarde-cli` 从最终源码构建，SHA-256 为 `2f094b8f16a8152d828bde534aa3719eb64cdf2626b63f62448686a2456b3c8c`。在 `/tmp/jarde-enum-delegate-projection-replay.2XgsEG/enum-constructor-delegation` 中复制证据目录后，仅改临时副本的 CLI SHA 校验并重放；仓库内冻结证据文件没有修改。原版与 JADX 在 `-g`、`-g:none` 下均通过 Java 8 编译及 `java -Xverify:all`，输出都为 `values=ZERO:0,ONE:1`、`effects=2:0,1`、`declared-constructors=2,3`。

冻结正例脚本原本把 Jarde 的公有 enum 文本放在名为 `jarde-DelegatingEnum-g.java` / `jarde-DelegatingEnum-g-none.java` 的文件中。更新 CLI 后，脚本的 javac exit 1 首先来自 Java 文件名规则，不能作为源码不合法的结论。验证时将两份相同文本分别复制为 `DelegatingEnum.java`，再以 `javac --release 8 -Xlint:-options` 编译，并以 `java -Xverify:all` 执行；两个 debug 模式都得到与原版/JADX 完全相同的三行输出。两份 Jarde 投影文本 SHA-256 都是 `c748f28b586e93a7763f83314a7ba9db7e21cf294d4846283de1fa1a4f68ded5`，两份 runner 输出 SHA-256 都是 `6851a69b1c53509cb546fbb30912b467f2f7da8c0107d5701c018475b4b40bf7`。九个冻结 verifier-valid 负例在同一临时副本及当前 CLI 下重放 9/9 通过，原/JADX 保持可编译、可验证执行，Jarde 均拒绝整组投影。

终端正文通过现有 emitter 计输出字节，类投影准备阶段再按完整常量、两构造器和可选初始化器文本计总输出；这与现有枚举 initializer RHS emitter 加类级投影总量计费相同。双构造器专门的 `output_bytes - 1` 用例验证耗尽时整组源文本回退到原物理呈现，没有只写一个构造器或一部分常量。预算/取消的完整来源、JSON item/outcome 与默认/完整证据比较仍属于 2.4；Root 的独立 3.1 验收也待进行，本记录不替代它们。
