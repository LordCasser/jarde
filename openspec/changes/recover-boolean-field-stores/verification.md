# 验证记录

实现任务 1.3、2.1–2.3 和 3.1 已完成。永久布尔字段 fixture 固定了两个 `Z` 字段描述符、11 个
`Code` 的实际 BLAKE3 哈希，以及从源 class 到补丁 class 的 SHA-256 证据。实现前基线仍为 40 行中
26 行不同。另有通过 verifier 的 accessor fixture，覆盖 verified `Z` 调用点和 callee put、各自的
BCI 与来源，并覆盖未验证 accessor 的拒绝路径。本次没有添加通用 int→boolean 转换。

Cargo 验证统一使用 `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1`：

- `cargo test --locked --test p3_boolean_field_stores -- --nocapture`：3 项通过，1 项忽略。
- `cargo test --locked --test p3_boolean_field_stores recovered_z_field_stores_match_the_patched_jvm -- --ignored --nocapture`：1 项通过；原始 source 与恢复出的完整 class 均成功编译，`java -Xverify:all` 执行结果与补丁 class 的 40 行输出完全相同。
- `cargo test --locked -p jarde-java --test p3_patterns a_verified_boolean_write_accessor_keeps_call_and_callee_put_origins -- --nocapture`：通过。
- `cargo test --locked -p jarde-java --test p3_patterns an_unverified_boolean_write_accessor_stays_a_call -- --nocapture`：通过。
- `cargo test --locked -p jarde-java --test p3_patterns`：50 项通过。
- `rustfmt --edition 2024 --check crates/jarde-java/src/build.rs crates/jarde-java/tests/p3_patterns.rs tests/p3_boolean_field_stores.rs` 与限定路径的 `git diff --check`：通过。

任务 3.2 和 3.3 留给 root 独立重放完整语料、运行相邻回归、更新 fingerprint 并完成 strict OpenSpec 验收。

## root 独立验收（3.2、3.3）

root 重建并冻结 CLI `/tmp/jarde-cli-boolean-root`，SHA-256 为 `8cd1f767ba8cde9928bacedc29263d44be8e5e61b46d3b6f00456fac1c7c77cd`。分别复制源、补丁器和审计脚本到 `boolean-field-stores/root-after-8cd1/` 与 `numeric-conversions/narrow-field-stores/root-z-after-8cd1/`，不覆写修前证据。最小样本 11 个 Code 未变，补丁原 class 与 jarde 完整输出 `javac`/`java -Xverify:all` 均为 0，40 行逐行相等、零引用；历史修前 26/40 错行仍保留。原始未补丁 Java 源与 Z 补丁语义不同，31/40 行不同是预期对照；JADX 对补丁完整类 `javac` 为 1，未声称其运行结果。完整 380 行样本 B/C/S 285 行及 Z 95 行全部与补丁 JVM 相同，整类 javac/runtime 为 0、无 `@bytecode` 引用。两次重放前后 CLI 哈希不变。

审读确认 `field_value` 仅在真实 `Z` 消费位置、且已有 boolean 证明不成立但渲染值为 B/C/S/I 时建 `% 2 != 0`；`-1 % 2 != 0` 成立，正负极值与低位一致。直接 put 与 verified accessor 共用该位置；赋值 origin 保留 put/call，accessor 派生保留 callee put，producer 子表达式仍在原位一次求值。未验证 accessor 继续 `jre_accessor_body` 拒绝，普通 boolean 字段路径不增加 `% 2`。这与 JVMS `putfield`/`putstatic` 对 `Z` 的低位规则相符。

root 另构造并用真实 JVM 验证了 accessor 栈形状：`boolean-field-stores/accessor-verifier-root/` 的 Java 8 class 仅将 `write(Z)V` 描述符改成 `write(I)V`，保留 `iload_1; invokestatic access$102(LAccessorVerify;Z)V` 和 callee `putfield f:Z` 的 Code；`java -Xverify:all` 通过，输入 -1、2、MIN、MAX 分别输出 true、false、false、true。补丁 class SHA-256 为 `2c97de3b4d12d927e1cb2e82a3cdd6b0f754723eb97f912a5c20c923323ff9d4`。这为 Rust accessor 测试的合法栈协议补了外部验证证据。

root 复跑 `cargo test -p jarde-java --locked` 全通过（109 unit、现有集成含 50 项 patterns）；`p3_array_access` 11、`p3_boolean_contexts` 13、`p3_boolean_field_stores` 3+1 ignored、`p3_deferred_value_order` 2+1 ignored、`p3_invocation_arguments` 3+1 ignored、`p3_narrow_field_stores` 2+1 ignored 通过，四项 ignored JDK 整类运行再单独通过。额外 `p3_narrow_integer_returns` 两项仍 RED：它们要求尚未实现的窄整数返回恢复，修前冻结 908c CLI 对 `directByte` 同样输出 Mixed 引用；不归入 Z 字段改动。

reader census 从 (104,683,81,236,8) 实测增长为 (105,694,81,236,8) 并复跑通过。fingerprint 初次只报告 `p3-boolean-field-stores` 八个新增文件，未报既有文件修改/删除；显式重生后 5 项通过、1 项生成器保持 ignored。`cargo fmt --all -- --check`、`git diff --check` 与 `openspec validate --all --strict --no-interactive` 54/54 均通过。本 change 的 9 项任务全部完成；`ireturn Z`、boolean 数组 `bastore` 及窄整数返回仍为独立规划边界。
