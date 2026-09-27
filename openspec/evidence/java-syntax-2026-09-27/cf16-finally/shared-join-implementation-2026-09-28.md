# CF-16 共用汇合点 finally：实现与复放

实现分支为 `codex/cf16-shared-join-finally`，基线 `origin/main` 的 `b42d02fd`。本报告只覆盖 OpenSpec `recover-shared-join-finally` 的 2.1–3.2；3.3 须由 root 独立复放。原固定类、JADX 固定 checkout 和修前首拒绝见 `fixed-test-triage-2026-09-28.md`。

## 证书和物理所有权

同一个私有 `Shape::SharedFinally` 现在有互斥完成合同：旧 `SavedReturns` 保持双保存返回的原证明；新 `Joined` 只接收当前实例 `Z` 字段的三份 `aload_0; iconst_1; putfield`，两条 `goto` 到同一续接。新路径逐项核三行异常表 `[5,10)→18 Exception`、`[5,10)→31 any`、`[18,23)→31 any` 的顺序与范围、具名和共用 handler 的 caught SSA、三份字段完整身份与 `this` 的入口 SSA、两个常量及 receiver 的唯一栈消费者、清理和跳转无额外异常保护、BCI 37–38 对原 throwable 的重抛。两个正常 successor 都必须唯一指向 BCI 39，且汇合点只能由这两条边进入；BCI 39–43 必须从同一实例读取同一字段并返回。

Region 的两个有界正文分别为 `[5,10)` 和 `[18,23)`；BCI 0–4 仍先写 `this.f=false`。共享证书拥有 BCI 0–38 的三个物理块，`Plan::join` 将 BCI 39 的块留给后续 Region。Builder 在原子 checkpoint 内输出一个 `finally` 字段赋值、一个具名 catch 和其后的 `return this.f`。完整方法唯一物理块 owner 为 `[0,18,31,39]`，集成测试逐个检查 26 个指令 BCI 都有来源，包括三份清理、两条 goto、handler 存储与重抛、独立的 join 读取和返回；证书记录三条异常行的 ordinal `[0,1,2]`。预算或取消时文本和 source map 同时为空。

## 固定正例和变体

`replay-acceptance.sh` 是修后的正向门，修前 `replay-baseline.sh` 仍保留为修前证据，不再用它判断成功。新脚本从固定 fixture 重建原 class，使用 pinned JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 的安装版 CLI 生成默认与 `--no-finally` 完整源码，再实时生成 Jarde 完整源码。四份完整 Java 类均以 `javac --release 8 -g:none` 重编、以 `java -Xverify:all` 执行；正常、具名 catch 和 `check()` 的 stdout 与原类逐字一致。默认 JADX 和 Jarde 各只有一次 `this.f = true;`；JADX `--no-finally` 有三次。Jarde 的 `test(Object)` 没有 `@bytecode` 或 explanation-only fallback，返回位于 `finally` 后。

固定原 class SHA-256 为 `24a23e9f96826d8a7ffe104ba31bfc0abb005141572920604f82e13b4b082807`；BCI 24 改 `iconst_0` 的 verifier 有效副本为 `0d2eb1586657e42667491485bcb000528c96344b2eb57d75c573b31e555face1`。另由 `SharedJoinVariants.java` 产生 verifier 有效的异常行缩短和第二条 goto 改入第一份清理的副本，SHA-256 分别为 `fcb1004a21c0bc10dbccd81065fd2f5f4934665ef9fea98bb7f005b1cb177e1d` 和 `78ac6c11ed43f56db389a9791dbaa6b14a5bea52ae202e1700ea24d6ac5b890f`。三种副本均通过 `-Xverify:all`，Jarde 均在物理 BCI 31 安全拒绝整个共享 finally 候选，未发布局部 finally。

`TestTryCatchFinallyError.java` 仅改变 `exc(Object)`，脚本逐条核它的 `test(Object)` 指令 BCI/操作与三行异常表仍和固定类相同。传入同一 `Error` 实例后，原 class 和 Jarde 重编 class 均在 catch-all 路径重抛**同一个实例**且 `f=true`。证书的 handler 路径只有一份物理字段写入，Jarde 完整源码也只有一份 finally 赋值；运行时字段是布尔值，单靠最终值无法独立计数写入次数，故次数结论依赖上述已核控制流与源码。

## 回归与边界

旧 `SharedFinallyCall` 和静态字段型 `SharedFinally` 的实时完整 Jarde 源码 SHA-256 仍分别是 `b5d33006e62fb6fbe6e93733b9af23503d4b5b9ab8d449f4a7537de1c98b7a12` 与 `e2e966dad66c5d55a6e2c84adb31c6820be16a9edc5cdc408ed68e5950a5092a`，与主线验收记录逐字节一致。两类原/Jarde 源码各自 Java 8 重编、`-Xverify:all`，正常和具名 catch 都为 `normal:1 / caught:1`。`cargo test -p jarde-java --tests --locked` 全过，包含既有单出口 finally、typed catch 正反例、shared-call/shared-field 证书，以及新正负例、来源、预算和取消测试。

`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`openspec validate recover-shared-join-finally --strict`、`git diff --check` 均通过。最后一次 `replay-acceptance.sh` 使用专用 `/private/tmp/jarde-cf16-shared-join-target`，退出时 `cargo clean` 清除 1232 个文件、约 1.8 GiB；额外一次显式 `cargo clean` 确认已无残留。清理后 `/private/tmp` 所在卷剩余约 67 GiB。未扩充保存返回、任意实例字段、多 catch 或通用异常重写；其它 CF-16 固定测试未宣称追平。
