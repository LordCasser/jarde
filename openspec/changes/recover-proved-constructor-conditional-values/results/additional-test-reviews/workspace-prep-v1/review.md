# 静态审阅

## 范围和输入

只读取当前 workspace manifests/文件、`/private/tmp/jarde-for-condition-workspace-root-v4.py`、其 v4 execution target inventory、旧 guarded build runner 与 target scanner。没有运行 Cargo/Rust/JDK/JADX/Git，没有执行候选 runner，没有修改 shared workspace、Git 状态或 `target`。

新输入闭包新增 8 个 fixture 文件和 1 个 Rust integration-test 文件：`tests/fixtures/java8/cf11-conditional-constructor-values/` 下 3 组 Java/class、README、pins，以及 `tests/recover_proved_conditional_constructor_values.rs`。用候选 `source_paths()` 对当前 workspace 做静态扫描（含源码内 include closure）得到 4727 个路径；这是本次观察值，不是 runner 的固定期望。runner 会重新扫描并冻结实际开始时的 pins，因此 root 后续更新 fingerprint 的内容会按真实值收录。

## 目标与命令数量

静态 target inventory 与 v4 保存的 Cargo metadata inventory 相比只增加 `jarde / recover_proved_conditional_constructor_values / test`，没有移除项。当前 manifests 没有显式 `[[test]]`/`[[bin]]` 等 target override；新增文件位于 root `tests/`，因此是一个自动发现的 integration target。

- targets：361，类型分布为 `{'bin': 1, 'example': 7, 'lib': 5, 'test': 348}`。
- test targets：{'jarde': 296, 'jarde-cli': 6, 'jarde-java': 41, 'jarde-query': 1, 'jarde-reader': 4}，合计 348。
- unit-target 批次：6（每个有 lib/bin/example/bench target 的 package 一条命令）。
- integration-test 批次：每 package 每批最多 4 个，`ceil(296/4)=74`（jarde）、`ceil(6/4)=2`、`ceil(41/4)=11`、`ceil(1/4)=1`、`ceil(4/4)=1`，合计 89。
- 测试批次：6+89=95；命令总量由 `fmt + metadata + tests + final clean` 动态计算，为 98。

runner 将 manifests/文件扫描结果与本轮实际 `cargo metadata --no-deps` target inventory 逐项比较，批次数从该 inventory 计算，不依赖 v4 的 360/95 常量。批次 raw stdout 中逐条保留 named outcomes 和所有 suite summaries；即使 test command 非零，也先把观察记录到 execution.json，再报告失败并走最终 clean。通过批次会核 outcomes 数等于 passed+failed+ignored 总数、summary 与 executable target 数对齐，并要求新 public tests 和新 unit tests 按完整名称 `ok`。

## Guard 与清理

继承了旧 validation guard 和 target-size scanner 的冻结 SHA。运行前要求 workspace `target/` 不存在且有 5 GiB 可用空间；每条子命令采用 process group 监测，1 秒轮询、`target` 上限 1 GiB。命令的 stdout/stderr 保留 raw 文件。每个已完成批次从 `target/debug/deps`（example 则 `target/debug/examples`）删除其 executable；最终无论测试成功与否都尝试 `cargo clean`，并核 `target` 消失、源 pins 不变。

Rust stack 使用环境默认值；按旧 runner 同样剔除 `RUST_MIN_STACK` 及 Java/classpath 注入变量，再使用 guard 的默认环境和固定 proptest seed。

## 静态审查结果与限制

候选通过 Python AST 和 symtable 语法/作用域解析。另将 outcome/summary 正则用于 v4 已保存 raw stream，仅确认解析器能覆盖当时每个 summary 的 passed/ignored outcomes；这不是当前代码验证，也不代表新构造器测试通过。

候选尚未运行，不代表全仓通过。root 执行前仍应核对当前输入与其真实 fingerprint 更新结果；runner 在启动时会自行动态 capture pins。
