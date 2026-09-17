# P1 验证维护记录

日期：2026-09-17。基线 `8fcdd66`；本记录描述其上的工作树变更，不冒充已提交版本或远端 CI 结果。生产 Rust 源码与两个 Cargo.lock 未改；CLI manifest 仅为既有 jarde path 依赖补精确版本，P2 只修订 OpenSpec。

## 1. 请求路由与反例

`fuzz/fuzz_targets/query.rs` 只调用共享 `exercise_query`。驱动 open 一次，依次运行 shape 0–4，逐次断言并释放报告；公开 query 错误不截断后续 shape。每次使用原有小预算，总工作量最多一次 open 加五次 query，不宣称是一个共享预算请求。

新增两项真实驱动测试：合法 CLASS/JAR 每种 shape 有非空结果、正确 relation，shape 4 明确报告 Verification/Debug 未实现；三个可打开的损坏种子每种 shape 都报告非 Complete execution 和 Partial。visited 必须为字面 `[0, 1, 2, 3, 4]`。

反例实测：临时把驱动循环换回首字节选择器后，`query_driver_exercises_every_shape_for_class_and_jar` exit 101，minimal-class 实际 visited `[2]`，期望 `[0, 1, 2, 3, 4]`。finally 恢复修复后，nightly corpus suite **9 passed / 0 failed / 0 ignored**，exit 0。

```sh
PATH="$HOME/.cargo/bin:$PATH" cargo +nightly-2026-07-20 test --manifest-path fuzz/Cargo.toml --locked
PATH="$HOME/.cargo/bin:$PATH" cargo +stable fmt --manifest-path fuzz/Cargo.toml --all -- --check
PATH="$HOME/.cargo/bin:$PATH" cargo +stable clippy --manifest-path fuzz/Cargo.toml --all-targets --locked -- -D warnings
```

以上最终均 exit 0。固定 nightly 未安装 rustfmt/clippy，因此格式和 lint 使用已安装 stable（rustc/cargo 1.88.0），未安装或升级工具链；编译测试与 sanitizer smoke 仍使用规定 nightly。

## 2. 本地 smoke

macOS Darwin arm64；rustc `1.99.0-nightly (9f36de775 2026-07-19)`；cargo-fuzz 0.13.2；libfuzzer-sys 0.4.13；Apple clang 21.0.0；默认 AddressSanitizer。两个 target 串行运行，分别从五个提交种子的全新临时副本开始，seed 固定为 17092026；不修改或积累提交语料。

在 `fuzz/` 中执行下式，target 分别为 query、artifact_tree，corpus 和 artifact_prefix 指向仓库外临时目录：

```sh
PATH="$HOME/.cargo/bin:$PATH" cargo +nightly-2026-07-20 fuzz run <target> <scratch-corpus> -- \
  -max_total_time=60 -max_len=65536 -rss_limit_mb=512 -timeout=10 \
  -workers=1 -seed=17092026 -print_final_stats=1 -artifact_prefix=<scratch-artifacts>/
```

| target | executed units | exec/s | coverage / features | peak RSS | 结果 |
| --- | ---: | ---: | --- | ---: | --- |
| query | 653513 | 10713 | 4404 / 9995 | 216 MB | exit 0，实际 61 秒，无 crash/契约断言失败 |
| artifact_tree | 1303696 | 21372 | 2760 / 5903 | 500 MB | exit 0，实际 61 秒，无 crash/契约断言失败 |

500 MB 距离 512 MB 只余 12 MB。保持限额并记录为后续观测项；此运行没有 OOM，不能据此断言余量充足或无内存泄漏。CI 仍各 20 秒且平台不同，本表不能代替远端 CI 的 peak RSS 或通过证据。smoke 不是完整安全/覆盖证明。

本机原始日志目录：`/var/folders/gm/lzn3ylzx4lz7mx29lftngdz40000gn/T/jarde-p1-harden-smoke-lr5h29su/`；旧路由反例日志：`/var/folders/gm/lzn3ylzx4lz7mx29lftngdz40000gn/T/jarde-p1-routing-nmfhnzfp/old-selector.log`。临时日志不入库，表中保留可复现命令和关键结果。

## 3. 生产边界与 P2 交接

本轮定向生产回归：`cargo test --locked --test p1_query_api --test p1_query_bounds --test p1_xref_code --test p1_xref_metadata --test p1_xref_bootstrap`，总计 **116 passed / 0 failed / 0 ignored**，exit 0。未重新运行完整 299 项双种子门禁或 JDK oracle；生产代码未变，这些历史证据保持归档原有口径。

P2 设计/三份 specs/任务已修订，20 项全部未勾选。后续从共享 facts、预算及结果契约开始，依次推进 Header resolver、raw/legacy CFG、Frame/SSA、Bytecode 产品；不在本轮实现 P2。历史 P1 归档未改。

补齐 CLI 的 path 版本约束后，`cargo +stable check --workspace --all-targets --locked` exit 0；本机 stable 即 Rust 1.88.0。根与 fuzz 的 fmt check 均 exit 0。

## 4. 两个依赖图与许可负例

CI 同一 supply-chain job 的两个 action step 分别指定根 `Cargo.toml` 和 `fuzz/Cargo.toml`，统一传入 `--all-features --workspace --locked --config ./deny.toml`。`--workspace` 明确包含根的 CLI member。CLI 既有 `jarde` path 依赖补 `version = "=0.1.0"`，保留 `wildcards = "deny"`；NCSA 从全局 allow 移到 `crate = "libfuzzer-sys@0.4.13"` 的精确版本例外。

已核对 [action.yml](https://raw.githubusercontent.com/EmbarkStudios/cargo-deny-action/v2.1.1/action.yml)、[entrypoint.sh](https://raw.githubusercontent.com/EmbarkStudios/cargo-deny-action/v2.1.1/entrypoint.sh) 和 [Dockerfile](https://raw.githubusercontent.com/EmbarkStudios/cargo-deny-action/v2.1.1/Dockerfile)：参数会传给 cargo-deny，目录切换在 subshell 内，配置仍从仓库根解析，工具版本为 0.20.2。精确 crate 版本形式见 [PackageSpec](https://embarkstudios.github.io/cargo-deny/checks/cfg.html)。本地使用相同 cargo-deny 版本，TOML/YAML 解析和双 manifest/config 参数检查均通过。

```sh
cargo deny --manifest-path ./Cargo.toml --all-features --workspace --locked --config ./deny.toml check
cargo deny --manifest-path ./fuzz/Cargo.toml --all-features --workspace --locked --config ./deny.toml check
```

| 用例 | 实测结果 |
| --- | --- |
| 根 workspace（含 CLI），最终 policy | exit 0；advisories/bans/licenses/sources 全部 ok，无 wildcard 告警 |
| fuzz workspace，最终 policy | exit 0；四段全部 ok |
| 临时 policy 显式拒绝 fuzz-only libfuzzer-sys | 根 exit 0；fuzz exit 2，bans FAILED，点名 libfuzzer-sys 0.4.13 |
| 临时 policy 删除 NCSA exception | fuzz licenses exit 4，NCSA rejected |
| 恢复最终 policy | fuzz exit 0；四段全部 ok |

反例只改仓库外临时 policy。原始日志在 `/tmp/jarde-harden-p1-validation.qQoKsb/`，最终证据为 `root-pass-v2.log`、`fuzz-pass-v2.log`、`reject-root-v2.log`、`reject-fuzz-v2.log`、`no-exception-fuzz-licenses-v2.log` 和 `restored-fuzz-v2.log`。有未命中的宽松许可/exception 提示，不把它们误写成无告警。

两个锁文件 SHA-256 与基线相同：

```text
Cargo.lock       a4fe17f1ae5842c0f07ba585731b6877f9ee16b4a91824ba1bf05f88f1e23787
fuzz/Cargo.lock  af5d0cb60db79176bdde87c4285c11e5ee49c9fb561a497c5140c234f4027384
```

## 5. 只读复核与交付状态

独立只读复核未发现 harness/回归测试的阻塞问题：确认五种请求实际执行、公开错误后继续、报告逐次释放、真实 target 与回归共用驱动，且未改生产 API。P2 规划机械复核确认三份 capability/spec 对齐、20 个任务均未勾选；这不替代未来各 IR 切片的语义复核。主 agent 另行核对 CI action 官方入口、最终严格 policy、正反例日志和 CLI 最小 manifest diff。

`openspec validate --all --strict --no-interactive`：**11 passed / 0 failed**；`git diff --check` 干净。P1 维护本地验证完成，P2 仍是规划；本轮尚未提交、推送或运行新的远端 CI，旧三个绿色 run 只证明原 P1 基线。
