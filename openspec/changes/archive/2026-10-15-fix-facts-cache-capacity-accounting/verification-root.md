# Root verification

Root 于 2026-10-15 完成本变更的本地 focused 复核。独立协议审查结果为 [protocol-review.json](/home/lordcasser/.cache/jarde-validation/core-integrity-20261015/cache-focused-v1/protocol-review.json)：状态 `accepted`、无审查错误，核对 4,952 项；8 条命令、16 条原始输出流；243 通过、0 失败、1 忽略。忽略项仍保持忽略，不计作通过。

本次实现 diff 共四个文件：

| 文件 | 变更内容 | execution 源码清单中的 SHA-256 |
| --- | --- | --- |
| `crates/jarde-reader/src/facts_cache.rs` | 增加共享总条目数 helper，三个层的准入和拒绝原因统一使用该计数；CP/Header 移除失效项时按实际移除条目的 weight 扣减。 | `74038dbb60a2f2888e4c8e535a3f1487552edeeb60477d31b8cc14fdd45aa800` |
| `crates/jarde-reader/src/facts_capacity_tests.rs` | 增加 reader 单元测试，覆盖六种插入顺序、零和满 entry/byte 边界、同 key 重写、format/registry/product 失效、仍存活的 payload 及混合 store。 | `19185d8f095286545f1690a9c0e5f4df938e295f6d96a7fca5110024be72e3a6` |
| `tests/p5_facts_cache.rs` | 增加公共路径测试，验证不兼容项移除后能在精确 byte 边界重新保留，并验证容量拒绝时仍返回完整 direct 结果。 | `5fbb01eaf6d180b4b7dac91ab278c34bc4ee569862189b1b78749cd6f5943801` |
| `tests/p5_benchmark.rs` | 只允许新增的 `cfg(test)` reader 模块构造私有 cache，同时保留生产代码构造边界检查。 | `818472fcc684063e4415a389dae5e90e863f3ae73dd4603090051e79b3bb3d17` |

`execution.json` 记录运行前后相同的 4,887 项源码清单；运行器文件及其 SHA 也保持不变。验收因此绑定到上表所列的源码快照。本验证记录和 task 勾选是在该次运行之后补写的。

回执记录 Rust stable 1.99.0、OpenSpec 1.11.0、默认 Rust stack（未设置 `RUST_MIN_STACK`）、单构建 job、关闭增量编译及串行测试线程。会构建产物的命令使用 5 GiB 最低可用空间与 1 GiB `target` 上限，并以 1 秒间隔检查、超限时终止进程组。最后的 `cargo clean` 收尾命令关闭资源阈值，以确保清理可以完成。8 条命令均成功退出：

1. `cargo fmt --all -- --check`
2. 当前 CI exact Clippy 命令：`cargo clippy --workspace --all-targets --all-features --locked`，使用仓库现有 lint allowances 并以 `-D warnings` 收尾。
3. `openspec validate --all --strict --no-interactive`
4. `cargo test -p jarde-reader --lib --locked`
5. `cargo test -p jarde --test p5_facts_cache --test p5_definition_read_reuse --test p5_shared_payload --test p5_container_lookup --all-features --locked`
6. `p5_benchmark` 中的生产 cache 边界测试 `the_engine_has_one_disabled_facts_cache_and_no_index_or_scheduler`，启用 all features 并使用 locked dependencies。
7. `cargo build -p jarde-cli --locked`
8. 最后执行 `cargo clean`；回执记录 `target_absent: true`。

reader 和 focused P5 测试覆盖所有插入排列、共享两条目上限及拒绝原因、零/满 byte 情形、同 key 重写、三种失效原因、无需 `clear` 即可重新填充、其他保留层与活动 `Arc` payload 的存活，以及 cache-on 与 direct 完整事实的一致性。生产边界测试还确认测试专用构造例外仅限于新增 reader 测试模块。

旧失败历史未与本轮成功拼接。特别是 root 没有重跑旧反例：先前三项失败是私有隔离运行，现已逐项核对原始输出：条目数 3 超过上限 2、移除后残留 185 bytes、混合 store 权重 1849 而应为 1664。归档 pristine 源码仅追加 `cfg(test)` 模块接线，便能逐字节重建旧运行的源码 SHA `b38cd5520ce3b67ce5cc14ae50e7cfd5339deb45656903b1e4c2656e2163ab0e`，确认原生产逻辑未改；这仍不是 root 复跑。完整历史和推导见 [private-history-review.json](results/root-focused-v1/private-history-review.json) 及其 `private-history.tar.gz`。本次验收只证明当前修复和这些回归测试。

本地实现及 focused 门禁已接受；本修复自身的提交和 CI 尚未完成，在线 CI 与最终交付仍待办，因此 task 2.2 保持未勾选。本结果不声称本地工具链与线上 CI 工具链完全相同，也不扩展到本 change 范围以外的完成状态。

已归档回执见 [root-focused-v1/README.md](results/root-focused-v1/README.md)。归档包 `capture.tar.gz` 的 SHA-256 为 `574c4d5b21d449d6bb038ced473f83eab0068ab99e728700e8d017c3cd1a1a28`。


## 2026-10-15 Final joint integration acceptance

以上局部记录与早期“待验”状态保留为历史，本节为最终验收。联合产品为 **d3e2b2457338b61660b842f5705b225bba08a73a**。cache自身CI38666595543与method自身CI38667091612已通过；canonical早期CI38668303129、emitter早期CI38669205597及c814 CI38672017137确实失败，均未改记成功。最终产品修正了真实P2计费golden、输出probe方法局部额度契约及corpus指纹，原bulk断言和Java/Runner/oracle未修改。

[联合证据归档](../2026-10-15-preserve-emitter-output-stop-reasons/results/root-joint-integration-v1/README.md)保存1771 payload，12721464字节，SHA-256 `f723dfa3e3e8bf3f91ed194453401464bfef295d474e20983abc0a5f91330073`，根重新打开1772成员逐字核对。

- focused-v3：462通过/0失败/2忽略，10命令/20raw；原bulk、完整P2/P5、fmt、当前CI Clippy、OpenSpec1.11.0及1.14.1 strict367/0、CLI build、finally exact clean通过。
- workspace-v3：195命令/390raw、730 suites；两seed各3587通过/0失败/98忽略，共7174/0/196，77具名门各一次通过；默认stack，守卫通过，target absent。本地Java25/javac23不冒称native CI。
- [自身CI38673548915](https://github.com/LordCasser/jarde/actions/runs/38673548915)：4 jobs/52 steps全成功，两seed各365 suites、3587/0/98；Rust1.99、Temurin25.0.4+7、Node22、strict367/0。根654checks和独立最终1173checks接受；不是把早期失败拼接成绿CI。
- 双JDK/debug-none/default-all完整matrix-v2：16 Jarde cells全部编译运行，baseline45/admission33行同原；JADX八baseline编译失败无runtime、八admission成功。矩阵实际用c814 CLI；d3重建CLI同59602720字节/SHA `cdf06694fafcb4e8cef5a4b9452676489e8cf02495c0254ce38a21e44ee4c7d5`，按字节恒同复用，不声称d3另跑16cells。

历史编译来源闭包4887项在运行前后恒同，且根按Git blob绑定到d3。额外13项fuzz遍历输入只在audit-time sidecar固定，不倒填为运行开始来源；实际corpus测试及根再生成核全2219项。最终doc/main HEAD另验来源、clean及target，不将产品CI说成未来文档提交自身CI。四项各4/4；后三项新计划仍0/27，CF11/CF18/71/612总体计数不增加。
