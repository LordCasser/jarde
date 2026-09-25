# Root acceptance

Root 重建 `jarde-cli` 后冻结 `/tmp/jarde-cli-static-root-after`，SHA-256 `8806d9aa06ad3b5fbcfe347144d09765dfbf3c9e172ee374eddf9313df893a44`。独立运行 `static-field-qualifier/post-fix/run_post_fix.py --cli /tmp/jarde-cli-static-root-after` 退出 0。主 fixture 721B/7Code、边界 1388B/16Code、接口 344B/3Code、属主 269B/3Code：每个完整类由原源码、JADX、Jarde 生成并通过 `javac --release 8 -g:none`、`java -Xverify:all`，逐行输出相同；主类 `write` 的接收者与 RHS 各执行一次。JVM 可执行的 CP-only 异属主补丁在 Jarde 中转为带 BCI 的保守引用，未错误重绑到 `Child.ping`；该拒绝正文不宣称可编译。

审查 `discarded_evaluations` 的唯一 `pop` SSA 使用与既有弃值归属、静态调用的接口/Java 类型/池属主判定，以及拒绝链的 producer/pop/call/consumer BCI，未见把副作用重复呈现或凭 opcode 邻接认领的路径。`jarde-java` 112+32+50 测试、静态限定符 4、meeting 6、static call 1、reference cast 5（1 JDK ignored）均通过。reader census `(123, 896, 98, 340, 8)` 与 314-class fingerprint（5 passed、1 ignored）通过；`cargo fmt --all -- --check` 与 OpenSpec strict 58/58 通过。磁盘 `target` 2.7G、可用 17GiB；未执行 cargo clean。

严格 Clippy 的既存 `region.rs:1736` `type_complexity` 仍 RED。仅豁免该 lint 时，`--all-targets` 还需要 `test-support` feature；带该 feature 又遇到独立 numeric-comparison 测试的 `useless_conversion`。`jarde-java`/`jarde` 库及本项 `p3_popped_static_qualifier` 测试在 `--features test-support -D warnings -A clippy::type_complexity` 下通过。`p3_type_qualifier::static_owners_survive_generated_parameter_and_suffix_names` 的既存 RED 已用修前冻结 CLI 重现，归属 `preserve-type-qualifier-bindings`，不混入本 change。
