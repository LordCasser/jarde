# 双终结布尔返回的 Java 8 对照与拒绝边界

所有源码以 OpenJDK `javac 23.0.1 --release 8 -g:none -Xlint:-options` 编译，class major 52。正例 `TernaryInIfProbe.class` 与原/JADX 对照位于 `openspec/evidence/java-syntax-2026-09-25/ternary-in-if/`，本目录只保存本 change 新增的控制。`TernaryInIfProbe-return2.class` 从正例唯一的 `04 ac 03 ac` 尾序列把首字节改成 `05`；`TernaryInIfProbe-int.class` 把唯一 UTF-8 方法描述符 `(LTernaryInIfProbe;)Z` 的末字节改成 `I`。两者长度不变，均经过 `java -Xverify:all`。各自的 `javap -c -v -p` 和八路径运行记录保存在同目录。

| class | SHA-256 | 边界 |
| --- | --- | --- |
| `TernaryInIfProbe-return2.class` | `f14ff75f4fcfe128b138d1b5a800a53b6c7e4d15aac96e6925d53adde25e1a0a` | `iconst_2; ireturn` 不是布尔真叶 |
| `TernaryInIfProbe-int.class` | `85501aaf06589a9732de94f502b66b2f8eb0d082d44706c2bdc59d86917e0473` | 方法返回描述符为 `I` |
| `EffectProbe.class` | `1e4b5b980fedd2e851e8d46a5c8a17881e3adf63f74202f3b3c211a431d855f9` | 两组测试之间有独立 `System.nanoTime(); pop2` |
| `BoundaryProbe.class` | `35a1e4716c8898cba3b9fff3430e279a6768f1386497541e74fdc13daf4ec040` | 第三终结出口、回边和受保护异常边 |
| `ExtraEntryProbe.class` | `4a6f8dd1b774211dda5b4f667929592289f18b37ce424a93c133179940494c40` | 内层双返回图根 BCI 8 有来自 BCI 1 与效果块 BCI 4–7 的两个普通前驱，候选不能以单入口认领 |
| `CountingProbe.class` | `9ae8a60ada0e454e9bf3435841b2fb56608cb4e0b82dcc8620d439762e6f9ee5` | 正向同形图，`CountedValue.equals` 记录动态调用次数 |

`ControlRunner` 通过反射执行两个字节补丁 class 与 `EffectProbe`，避免把原 `Z` 调用点误用于 `I` 方法。`BoundaryRunner` 执行三个边界方法。`CountingRunner` 的八行 `值:equals调用次数` 为 `true:0`, `true:2`, `false:0`, `false:1`, `false:1`, `false:1`, `false:2`, `false:2`；永久测试同时重编并运行原类与 Jarde 完整类，逐行比对这八行，验证惰性次序。各 class 的 BLAKE3 和字节数还登记在 `tests/fixtures/corpus-fingerprint.json` 的本目录条目。

2026-09-25 定向结果：`p3_two_exit_return` 七项通过；相邻的短路返回、字段短路、普通 `if`、Region fallback 来源、owner overlap、转接 gateway 六套测试共 17 项通过、一个需 JDK 的旧字段测试保持 ignored。`cargo fmt --all -- --check` 与 `openspec validate recover-shared-terminal-boolean-returns --strict` 通过。`cargo clippy -p jarde-java --all-targets` 退出零但报告仓库既有 18 条警告；加 `-D warnings` 因这些既有警告失败。共享 `p5_corpus_fingerprint` 中本目录条目均已登记，但整个测试仍因另 121 个与本 change 无关的 corpus 文件未登记而失败，没有代填它们。

定向执行：`cargo test --test p3_two_exit_return --locked`。手动重放控制时，把两个补丁 class 分别复制为 `TernaryInIfProbe.class` 到不同临时目录，连同 `ControlRunner.class` 运行 `java -Xverify:all -cp DIR ControlRunner TernaryInIfProbe`；`EffectProbe` 使用同一 Runner 的 `EffectProbe` 参数。`BoundaryRunner` 与 `CountingRunner` 可直接在本目录用 `java -Xverify:all -cp tests/fixtures/p3-two-exit-return NAME` 运行。
