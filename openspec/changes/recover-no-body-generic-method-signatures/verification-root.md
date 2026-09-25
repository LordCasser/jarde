# Root 独立验收（2026-09-24）

`recover-no-body-generic-method-signatures` 的 1.3、2.1–2.3 已由 Luna 子 agent 实现；本记录是 root 在其结束后对共享工作树的独立审查，不把 agent 的测试结论直接视为验收。

审读 `src/class_source.rs` 与 `src/facade.rs`：`NoBody` 方法现先走 reader 同一方法 Signature 作用域及 descriptor/Exceptions 逐位置擦除证明；只接受顶层、Object 物理父类、零接口的抽象方法，并拒绝 Object 同名和本类 `Methodref` 绑定未证的形状。类头的变量 scope 仅在完整类头投影成功后传入。方法变量、参数、返回及异常从结构树一次拼写，经输出预算后由 `project_generic` 原子替换；无 Signature 异常后缀仍用物理 `Exceptions`。异常类型变量需第一界为确定的四个 JDK Throwable 根，类异常项只接收单段、无类型实参的可拼写名字。有正文方法仍走原候选路径。

独立执行 [`no-body-generic-methods/replay.py`](../../evidence/java-syntax-2026-09-24/no-body-generic-methods/replay.py) 退出 0：抽象类与根接口的 `-g`/`-g:none` 四种完整类均 Java 8 重编，覆写/实现与强类型调用方重编、`-Xverify:all` 执行及泛型反射与原 class 相同。新 Jarde 抽象类两份源码 SHA-256 均为 `2887e4dded4ca9ece35d2547da05b159c8ac4751802bac6a3bc16e3a21bafa60`；根接口两份均为 `f31528c0ac93dc0bb38eb21753c5b8c4fa868c72a6f8b2424e01a37f2bcb543e`。相同根接口下，JADX 1.5.6 将 `throws X` 写成 `throws Exception`，调用方 `javac` 退出 1。独立执行 [`method-local-generic-throws/negative/replay.py`](../../evidence/java-syntax-2026-09-24/method-local-generic-throws/negative/replay.py) 退出 0：未绑定变量、物理异常擦除不符、`Exception<String>` 异常项及未知父类契约均是 JVM 可加载/验证的输入，Jarde 分别给出局部拒绝；物理声明 Java 8 重编与验证通过。脚本对这些结果均有断言。

同一私有 Cargo target 的 `class_source` 47/47、`field_generic_projection` 3/3、`generic_method_projection` 9/9、`generic_throws_projection` 8/8、`ordinary_generic_projection` 14/14；reader Signature 15/15、query 库 7/7。`cargo clippy -p jarde --test generic_throws_projection -- -D warnings` 加仓内既有五项 lint 豁免通过；`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-no-body-generic-method-signatures --strict` 通过。预算/预取消对新的 `<X>` 路径有独立定向测试，未发布半个泛型头。

界限：带继承契约、同类调用、非 JDK 根界的异常变量与有正文方法不在本 change 的准入范围；这些局部拒绝不等于全语法支持。私有测试 target 及实现 agent 的私有 target 已清理。
