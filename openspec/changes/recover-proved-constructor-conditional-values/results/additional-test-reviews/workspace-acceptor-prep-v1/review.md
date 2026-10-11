# 静态审阅

验收器以 `/private/tmp/jarde-for-condition-local-workspace-accept-luna-v2/accept_workspace.py` 为基础，沿用 committed `verify-cf11-own-ci.py` 和 `verify-conditional-ci-root-v5.py` 的解析接口。固定 runner 为 `/private/tmp/jarde-constructor-conditional-workspace-root-v1.py`，SHA-256 `81484f984cdf909980b7dab25ee38ac560de0a1fa6028dc912b82ca05e0f0606`。验收器不调用 Git；它只读 receipt、metadata、命令 raw streams 和当前 live source pins。

期望 schema 为 `constructor-conditional-full-workspace-batched-root-v1`，source closure 4727、metadata targets 361、batch commands 95、总命令 98。它根据 metadata 与 committed helper 重建每条批次 argv，再核每个 target 的 compiler artifact、stderr `Running` header、stdout test block 与 runner convenience fields。所有 196 个 raw streams 必须位于 evidence 目录、非 symlink，字节长度和 SHA-256 与 execution receipt 一致；metadata 文件必须与命令 1 stdout 原字节相同。

source pins 必须与 `workspace_input_paths()` 返回的完整 4727 路径集合一致，并满足 before、after、live 三者 hash 相等。这样当前 corpus fingerprint 以真实运行时字节参加闭包。验收还核对 seed `5350648285461741569`、默认 Rust stack、5 GiB 可用空间、1 GiB target 上限、1 秒 guard 轮询、每条命令 exit=0 和 guard 未停止。

required test 名称与当前 runner 的 `REQUIRED_NAMED_TESTS` AST 常量逐项相同，共 12 个：

- `tests/recover_proved_conditional_constructor_values.rs` 的 5 个 public tests；
- `field::tests::conditional_new_receipt_counts_only_one_method_local_direct_new`；
- `field::tests::conditional_new_receipt_rejects_derived_primary_and_derived_only_candidate_anchors`；
- `init::tests::conditional_constructor_output_has_one_live_stack_phi_and_one_join_consumer`；
- `init::tests::actual_local_conditional_new_has_its_local_store_as_the_single_consumer`；
- `init::tests::actual_shared_field_return_keeps_dup_outside_the_new_expression`；
- `init::tests::actual_nested_conditional_constructor_arms_do_not_become_one_two_way_site`；
- `init::tests::actual_independent_branch_effect_stays_on_the_existing_statement_path`。

public tests 通过 metadata 的精确 integration source 映射到 target；两个单测源文件通过 helper 映射到 `jarde-java` lib target，并核其 module 在 crate `lib.rs` 中声明。每个测试函数定义必须唯一，raw block 中对应完整测试名必须恰好出现一次且为 `ok`，runner 自己的 required-name/pass-name 清单也必须精确等于上述 12 个名称。

每批 runner 保存的命名 outcome 与 summary raw lines 必须逐行等于 stdout；helper 对 raw blocks 独立得到的逐 target outcomes/counts 必须相等，且聚合后等于 runner 的 per-batch/workspace totals。验收清单的 totals 来自当前 361 个 raw observations，没有固定的历史通过数。所有完成 target 的 executable removal receipts 必须唯一、在 `target/debug/deps` 或 `target/debug/examples` 下且数量为 361。最终命令 97 必须是成功且受 guard 保护的 `cargo clean`，cleanup receipt 一致、target 缺席、source pins 不变。

候选通过 Python AST 与 symtable 静态解析。准备时读取到 root execution 仍是 `running`、只有 fmt 和 metadata 两条记录；未把它当成完成结果，也没有执行 acceptor 或 runner。
