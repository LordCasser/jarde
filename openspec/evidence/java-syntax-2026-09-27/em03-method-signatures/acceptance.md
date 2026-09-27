# EM-03 首片验收（2026-09-27）

基线提交为 `33f15fde5847ff150bf6689fc990866afd08bc13`。本次只投影无 LVT、完整且与 descriptor 位置一致的成员级 `MethodParameters`；名称通过同次 `RecoveryFacts` 进入正文和声明，`final` 仅随完整投影出现。`Exceptions` 继续走既有声明读取。

固定 [replay.py](replay.py) 使用 `javac 23.0.1 --release 8 -g:none -parameters` 与 OpenJDK `23.0.1` 的 `java -Xverify:all`；JADX 检出为 `2fb1b16386941660fda07e9017285aec40fcb37f`，其五份测试的 SHA-256 见 [summary.json](accepted/summary.json)。本次 Jarde CLI SHA-256 为 `db0c3e9c831a22b11a9a912f202b1cdc675e56716821dc74da6de0f9a8a8a160`。三份完整源码均通过重编和验证，运行六行逐字一致：`ok`、`9`、`negative`、`paramStr:false:number:true`，以及两行 `[class java.io.IOException]`。生成源码、`javap`、编译及运行日志均在 [accepted](accepted/) 中。

回归结果：`jarde-reader` 178/178、`jarde-java` 220/220、`tests/class_source.rs` 84/84、`p3_declarations` 5/5、`generic_method_projection` 16/16；`cargo fmt --check`、`cargo check --workspace` 和 `openspec validate recover-proved-method-parameters --strict` 通过。集成测试另外覆盖无属性的槽位名、有 LVT 的旧路径、实例接收者与 `long/double` 双槽、宽槽名称碰撞、空名、重复名、项数不符和非法 flags；reader 测试覆盖完整长度、重复属性、无效索引/flag、一次属性字节计费、低预算及取消。

边界保持独立：LVT 与 `MethodParameters` 的通用冲突协调、隐式或合成参数重排、参数注解重索引、Smali 畸形 `Exceptions` 和泛型签名不由本首片证明。`cargo test -p jarde --lib --no-run` 被主线已有的 `member_inner` 测试调用缺参阻断；本分支不修该问题，详见 [基线测试债务](baseline-test-debt.md)。
