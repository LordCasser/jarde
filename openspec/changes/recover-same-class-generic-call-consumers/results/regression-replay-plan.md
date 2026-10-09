# Candidate regression replay plan

本文记录下一版候选 CLI 可复用的命令和现有对照结果。候选可执行文件尚未就绪。基线输入已独立复核：accepted-cli-v5 将冻结的 140-input 集合重放到新目录，完整结果索引如下。运行候选回归前，将 `CANDIDATE_CLI` 和 `CANDIDATE_SHA256` 设为最终可执行文件路径及独立测得的 SHA-256。

当前对照 CLI 为 `/tmp/jarde-raw-receiver-final-v3-cli`，SHA-256 为 `3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70`。它是新 GC01–08 验收语料、80-constructor 和 64-raw 回归重放所用的已验收 raw-receiver 基线。较早的 raw CLI `3e7241e99b0068ede015b4dbc419212f5a4f63cb9a78f37429662b34f6d7a48c` 仅作为历史对照；不要将其用作当前基线。

## GC01–08: frozen 140-input corpus

冻结输入位于 `openspec/evidence/same-class-generic-call-consumers-2026-10-09/frozen-inputs-v1`，源文件和 jar 的哈希记录在 `frozen-input-manifest.json`。基线重放结果位于 `openspec/evidence/same-class-generic-call-consumers-2026-10-09/results/baseline/accepted-cli-v4`。其中有 140 个唯一 `(family, JDK, debug)` 输入：35 个族覆盖 Corretto 8u432 和 OpenJDK 23.0.1，每个 JDK 均进行 debug 和 no-debug 编译。原始源码编译及 Probe 通过 140/140；JADX 通过 130/140；已验收 CLI 完整类编译通过 72/140。这些只是编译计数，不代表整体验收：每个编译成功的结果还分别包含 `-Xverify:all` 行为 Probe 和结构化泛型反射结果。基线结果没有 Probe 失败；候选的行为和声明应与 `original` 对照，并分别报告编译、Probe、API 匹配及失败数量。

运行候选时显式指定 candidate 标签，避免结果写入 baseline flavor：

```sh
python3 openspec/evidence/same-class-generic-call-consumers-2026-10-09/replay.py \
  --out openspec/changes/recover-same-class-generic-call-consumers/results/candidate/gc01-08-v1 \
  --frozen-inputs openspec/evidence/same-class-generic-call-consumers-2026-10-09/frozen-inputs-v1 \
  --cli "$CANDIDATE_CLI" --cli-label candidate
```

索引应包含 `manifest.json`、`summary.md`，以及每例的 `compile-sources/candidate/candidate-javac.{stdout,stderr}` 和 `candidate-probe.{stdout,stderr}`。manifest 记录候选 CLI 哈希、冻结 manifest 哈希、JDK/JADX 哈希、精确命令、源文件和 class 哈希、声明头、Probe 输出及反射记录。当前已验收 CLI 基线为 `openspec/changes/recover-same-class-generic-call-consumers/results/baseline/accepted-cli-v5`。其原始源码编译+Probe 为 140/140；JADX 编译+Probe 为 130/140，其中反射匹配 126/130；baseline 编译+Probe 为 72/140，其中反射匹配 20/72；通过的 Probe 均无失败。manifest 包含 140 个唯一 key、35 个族、全部四种矩阵模式，3,657 个结果文件哈希已独立复核。结果中没有保留 `.class` 文件。实际执行的 runner 保存为 `accepted-cli-v5/replay-executed.py`，SHA-256 为 `6b102b7c16e0bc318d42d3fbd21a32d51bd3bbeeeb72c8f66722808c09f28960`。

v4 的输入和结果测量仍然完整，但其 manifest 中的 runner 路径最初指向可变的 evidence 脚本。该文件变更后，无法从 workspace 找回与旧 runner 字节完全一致的副本。v4 manifest 未被改写，不应将其描述为保留了不可变 runner 快照。v5 在创建输出时保存并计算实际执行的 runner 副本哈希，补上了这一来源记录缺口。

## GC09 field-write replay: 23 families

现有 runner 为 `openspec/changes/recover-class-scope-constructor-parameters/results/field-regression/replay.py`；其固定输出位置是 `results/field-regression/root`。应通过一次性小 wrapper 调用：在执行 `main()` 前，将模块的 `RESULTS` 全局变量设为新的输出目录。不要将其指向固定的历史输出目录。原 runner 接受 `--baseline` 和 `--candidate`，每条结果都会报告两个 flavor。外层运行记录必须包含 runner 源文件哈希、argv、基线和候选可执行文件哈希，以及候选输出标签。

该协议覆盖 Corretto 8u432 和 OpenJDK 23.0.1（`-g:none`）上的 23 个源代码族，每个 CLI 对应 46 行。当前已验收 CLI 的结果位于 `openspec/changes/recover-class-scope-constructor-parameters/results/field-regression/root/summary.json`：基线和先前候选都在 44/46 项中编译成功且行为保持一致；两个 `SCGB` 行是有意拒绝源码编译的控制。基线 CLI 的完整反射匹配为 18/46，先前候选为 20/46。将这些已有诊断和边界作为对照；不能把 CLI exit 0 当成编译通过。

23 个族由 14 个 generic-holder 字段用例、2 个 root anchor 和 7 个 root Probe 组成。adapter 按 JDK leg 和族复用这 14 个历史 generic-holder jar；其 source SHA-256 与先前验收 manifest 一致，每个 jar 只包含同名 class。adapter 提取这些冻结 class 供 original runtime 使用，并按字节复制 jar 作为 CLI 输入，不重新编译这些源文件。其余九项是没有对应历史输入 jar 的 source-only anchor/Probe；只有这九项会编译到临时 classes 和 jar。adapter 记录历史 jar SHA-256 和 class 哈希、源文件哈希、实际 javac 参数，以及临时 jar/class 哈希；不会持久化新 fixture 输入。

使用 adapter 包装现有 runner。adapter 不改历史 runner；它保存并执行不可变的 runner 副本，同时保留原来的 `__file__` root，校验两个 CLI 哈希，将结果重定向到新目录，并为每次 javac 调用注入显式的空 classpath/sourcepath 目录。无副作用 stub 会检查实际 `main` 函数是否读取重定向后的输出路径和 `run` hook。adapter 记录自身及 runner 快照、源文件、命令和输入 jar/class 哈希。

```sh
python3 openspec/changes/recover-same-class-generic-call-consumers/results/field-23-replay-adapter.py \
  --baseline /tmp/jarde-raw-receiver-final-v3-cli \
  --candidate "$CANDIDATE_CLI" --candidate-sha256 "$CANDIDATE_SHA256" \
  --out openspec/changes/recover-same-class-generic-call-consumers/results/candidate/gc09-field-23-v1
```

adapter 会拒绝已存在的输出路径，并将摘要保存到 `gc09-field-23-v1/root/summary.json`，同时保留每例源文件、JSON 报告、javac 输出、runtime 输出和反射差异。旧 runner 自身的行标签是 `baseline` 和 `candidate`；`run-metadata.json` 将这些标签对应到可执行文件路径和哈希。其中 14 条 `original-compile` 记录表示提取了字节完全一致的历史 JAR class，详情见 metadata；其余九条记录保留真实 javac stdout/stderr 和命令参数。

## GC09 constructor replay: 80 inputs

复用 `openspec/changes/recover-class-scope-constructor-parameters/evidence/replay.py` 及其 `evidence/manifest.json` / `checksums.sha256`。其中 20 个族覆盖两个 JDK 和两种 debug 模式，共 80 个输入。历史构造器结果对使用的 CLI 哈希为 `34a5288badf6fb40020c5117ef749ae4705545b29ee35a9fefda48de24276b53`（baseline）和 `3e7241e99b0068ede015b4dbc419212f5a4f63cb9a78f37429662b34f6d7a48c`（candidate）。该结果对中，原始源码 80/80 编译并执行成功；两个 Jarde CLI 都在 72/80 项中编译并保持行为一致。八项源码编译失败来自 `CallHold` 和 `ExceptionHold` 的四种 JDK/debug 组合：`Object` 被传给已发布的 class 变量 `T`。旧 baseline 的完整反射匹配为 0/80、构造参数匹配为 8/80；旧 candidate 的完整反射匹配为 36/80、构造参数匹配为 48/80。

本 change 使用较晚验收的 raw-receiver CLI 作为对照可执行文件：`/tmp/jarde-raw-receiver-final-v3-cli`，哈希为 `3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70`。它的 constructor-80 指标尚未单独索引，因此下面的命令会将它作为 `baseline` flavor，与新候选一起重放。不要把旧构造器 CLI 的反射数量记作 3f75 的测量结果。`CallHold` 和 `ExceptionHold` 仍是本 change 要修复的 GC06 构造器调用用例。`SCGB` 是 field-23 控制项，不属于 constructor-80 族。

```sh
test "$(shasum -a 256 /tmp/jarde-raw-receiver-final-v3-cli | awk '{print $1}')" = 3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70
test "$(shasum -a 256 "$CANDIDATE_CLI" | awk '{print $1}')" = "$CANDIDATE_SHA256"
python3 openspec/changes/recover-class-scope-constructor-parameters/evidence/replay.py \
  --baseline /tmp/jarde-raw-receiver-final-v3-cli \
  --candidate "$CANDIDATE_CLI" \
  --out openspec/changes/recover-same-class-generic-call-consumers/results/candidate/gc09-constructor-80-v1
```

runner 只接受 `--baseline`、`--candidate`、`--out` 和可选的 `--only`；它会创建新输出目录并记录冻结 manifest/checksum 哈希。在外层运行索引中，将已验证的两个 CLI 哈希记在该结果路径旁。保留所有编译和运行 transcript。必须完成现有状态检查和全部四种 JDK/debug 组合，才能判定该族通过。完整验收应将新候选与刚测得的 3f75 基线及原始源码比较，不能只与较早的历史结果对比较。

## GC09 raw-receiver replay: 64 inputs

复用 `openspec/evidence/raw-receiver-field-selection-2026-10-09/replay.py` 及其冻结输入。它覆盖 16 families × 2 JDKs × debug/no-debug = 64 rows。当前已验收 CLI 的归档对照结果为：行为 64/64、字段泛型反射 60/64、方法 API 52/64、类 API/bounds 60/64。之前的 baseline CLI 结果为：行为 64/64、字段反射 12/64、方法 52/64、类 API 60/64。已知的 `InstanceRawLocal` 候选拒绝仍是四种组合中的显式边界。

该 runner 每次调用只接受一个 CLI，因此应将已验收基线和候选分别运行到新的结果目录：

```sh
python3 openspec/evidence/raw-receiver-field-selection-2026-10-09/replay.py \
  --cli /tmp/jarde-raw-receiver-final-v3-cli \
  --cli-sha256 3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70 \
  --label accepted-baseline \
  --out openspec/changes/recover-same-class-generic-call-consumers/results/candidate/gc09-raw-64-baseline-v1

python3 openspec/evidence/raw-receiver-field-selection-2026-10-09/replay.py \
  --cli "$CANDIDATE_CLI" --cli-sha256 "$CANDIDATE_SHA256" \
  --label candidate \
  --out openspec/changes/recover-same-class-generic-call-consumers/results/candidate/gc09-raw-64-candidate-v1
```

每次运行都会记录准确的 CLI/JADX 哈希、源文件和 JAR 哈希、命令、stdout/stderr、编译结果、行为标记及反射身份。不要将冻结 evidence 目录用作 `--out`。

这些重放不包含 Cargo 命令。候选验收需要完整的 GC01–08、constructor-80、field-23 和 raw-64 结果，并分别报告源码编译、runtime 行为和 API/反射计数；CLI 命令成功本身不构成验收结果。

## Nested call-result argument supplement

design 接受将一次调用结果作为同类泛型调用的参数。该额外源码与冻结的 v1 语料隔离，不改变其 140 个输入的计数。源码为 `NestedCallArgument<T>{first(T), second(T), relay(T x){return second(first(x));}}`；独立的四种组合结果位于 `openspec/changes/recover-same-class-generic-call-consumers/results/extension/nested-call-argument-v1`，由 `results/nested-call-argument-replay.py` 生成。

在全部四种 JDK/debug 组合中，original 和 JADX 源码均编译成功，并通过 12 项 Probe 检查：每个方法的参数和返回类型都解析到目标类准确的 `T` 声明；`relay` 经两次嵌套调用后返回同一个 marker 对象。已验收基线 CLI 在四种组合中生成的源码都未通过 javac：嵌套调用结果被擦除为 `Object`，随后传给要求 class `T` 的调用。extension manifest 保留 original/JADX/baseline transcript、生成的输入 JAR、class 哈希、命令和 runner 快照。extension manifest 的文件哈希与 runner 快照已独立检查；没有遗留 `.class` 文件。

候选 CLI 就绪后，使用 `results/nested-call-argument-candidate-replay.py` 只重放这四个冻结 JAR。入口会检查冻结 extension manifest 和每个输入文件的哈希，不会调用 javac 或 jar 重建输入；它要求 candidate 标签和匹配的 CLI 哈希，并在新输出目录保存不可变 runner 快照。入口会在空搜索路径下使用冻结的 `NestedCallProbe.java` 编译候选完整类，运行时仅使用新生成的 candidate classes 并启用 `-Xverify:all`，还要求结果中能看到全部 12 项泛型声明和 marker 检查。输出目录与 extension v1 分开：

```sh
python3 openspec/changes/recover-same-class-generic-call-consumers/results/nested-call-argument-candidate-replay.py \
  --frozen-inputs openspec/changes/recover-same-class-generic-call-consumers/results/extension/nested-call-argument-v1 \
  --cli "$CANDIDATE_CLI" --cli-sha256 "$CANDIDATE_SHA256" --label candidate \
  --out openspec/changes/recover-same-class-generic-call-consumers/results/extension/nested-call-argument-candidate-v1
```
