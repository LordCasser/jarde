# 实现完成复核（2026-09-20）

**结论：本次 review 与规划修订完成，任务链实现暂不能确认收尾完成。** 历史阶段和七项后续交付的归档保留；当前行为仍有 R1/P1 与 R2/P2 两条停止语义缺口，由 [preserve-task-operation-stops](changes/archive/2026-09-20-preserve-task-operation-stops/proposal.md) 单独修正。`optimize-demand-workloads` 仍是 0/22 的调查专项，O1 子交付已完成不等于专项已完成。

## 范围与证据基线

- 审查 HEAD：`bafdcecd36dfd7a71e2c28a4eb43c4097ff61bff`；最后行为提交 `85828c4ae2b3ebf1e6d2dc9c822ee768bbf002a2`。开始时工作树干净；本次只改规划/文档，不修改生产实现，不提交或归档。
- 重点审查近期导航、任务库操作与 CLI 的身份交接、选择和停止传播；对照主 specs、归档验证、当前性能计划及源码。全量门禁覆盖 workspace，但不宣称穷尽所有 JVM 输入与算法。
- Atlas 打开成功，但当前事实库查询新方法为空，目录查询报 `failed to validate type ranges for src/call_context.rs`；未重建全库，以下行为结论来自当前源码与实际二进制反例。
- 反例通过 `cargo build -p jarde-cli --locked` 构建的 CLI 验证，其 JSON 是库报告直接序列化；仅解析已提交受控 fixture，不执行目标代码。

## R1 / P1：不完整名称搜索被当作唯一或缺失

位置：`src/facade.rs::bind_method`（3842–3864）、`bind_class`（3732–3758）；共享搜索为 `search_named_classes`。这是 `2428752` 新任务操作路径的问题，不重开旧 R8/R9。

`bind_method` 根据 `report.candidates.len()` 直接决定未找到/执行/歧义。零和单候选分支均丢弃搜索的 execution、coverage 与 diagnostics；候选集只是已确认前缀，不能证明唯一或缺失。`bind_class` 零候选也误报缺失，单候选虽保留搜索停止，仍允许后续 body 工作。

受控 JAR 用已提交的 `NestedEval.class`，加一个字节为 `broken` 的同名路径候选；请求 `nestedPlain(I)I`：

| 输入/顺序 | 当前观察 | 应有结果 |
| --- | --- | --- |
| 仅有效 `NestedEval.class` | recover exit 0、Complete、1 body | 完整唯一的正向对照 |
| 有效 `NestedEval.class` → 损坏 `x/NestedEval.class` | recover exit **0**、`performed`、Complete、1 body；无候选损坏诊断 | 未完成选择，保留候选与损坏证据，不执行 body，CLI exit 4 |
| 损坏 `NestedEval.class` → 有效 `x/NestedEval.class` | recover/class-view exit **2**、`operation_target_not_found`；后者称只检查 1/2 候选 | 未完成选择，不宣称不存在，CLI exit 4 |

对第二行运行 class-view，可看到 `classfile_decode` 失败诊断与 Failed 搜索，证明损坏是实际发生且可报告的；recover 则丢掉该证据。仅给后续分析补一个状态不够，唯一性未证实前不能自动执行。

## R2 / P2：类视图顶层忽略方法体停止

位置：`src/facade.rs::class_view`（843–866）；`ClassViewReport` 文档承诺汇总 every body-level stop（2879–2881），实际只把 body 压入集合。`crates/jarde-cli/src/task.rs::class_view_plane`（1026 起）另行遍历补算，所以 CLI 已退出 4，直接库调用仍读到顶层 Complete。

将 fixture 的 `nestedPlain(I)I` 首 opcode（class offset 311）由 `0x1a` 改为非法 `0xff`，同时请求 `nestedPlain(I)I` 与未损坏的 `nestedLocal(I)I`：

- 顶层 `execution.status = complete`。
- 第一个 body 为 `Read`、execution Partial，`stopped_at = instructions / BCI 0 / classfile_instruction_decode`；第二个 body Complete。
- CLI exit 4；类/成员的 `artifact_structural` coverage 为 `complete_within_schema`，该结构覆盖本身合理，不应为了汇总 body 停止而改写为未扫描。

应由库汇总顶层 execution，同时保留正常 body 和每个平面的真实覆盖。已归档 CLI verification 曾披露此边界；披露不能替代库契约的闭环。

## 最小复现

从仓库根目录运行；只创建临时文件。固定 fixture SHA-256，损坏 offset 也作前置断言。输出打印退出状态与完整/停止摘要，原始 stdout/stderr 保存在显示的临时目录中。

```sh
cargo build -p jarde-cli --locked
python3 - <<'PY'
from pathlib import Path
import hashlib, json, subprocess, tempfile, zipfile

out = Path(tempfile.mkdtemp(prefix='jarde-review-'))
fixture = Path('tests/fixtures/p3-nested-eval/v8/NestedEval.class').read_bytes()
assert hashlib.sha256(fixture).hexdigest() == '141c3dcb3990605466dd54eb1a9bcc1942d0907e4e32d8d0587eb443f0c5c2e9'

def run(label, path, args):
    p = subprocess.run(['target/debug/jarde-cli', *args, '--input', str(path),
                        '--format', 'json'], capture_output=True, text=True)
    (out / (label + '.stdout.json')).write_text(p.stdout)
    (out / (label + '.stderr.json')).write_text(p.stderr)
    d = json.loads(p.stdout or p.stderr)
    execution = d.get('presentation', {}).get('execution', d.get('execution', {}))
    print(label, 'exit', p.returncode, 'execution', execution.get('status'),
          'error', d.get('error', {}).get('code'),
          'bodies', [(b['kind'], b.get('execution', {}).get('status'))
                     for b in d.get('bodies', [])])

recover = ['recover', '--class-name', 'NestedEval', '--method-name', 'nestedPlain',
           '--descriptor', '(I)I', '--policy', 'plain-jar']
for label, entries in [
    ('valid', [('NestedEval.class', fixture)]),
    ('bad_after', [('NestedEval.class', fixture), ('x/NestedEval.class', b'broken')]),
    ('bad_before', [('NestedEval.class', b'broken'), ('x/NestedEval.class', fixture)])]:
    path = out / (label + '.jar')
    with zipfile.ZipFile(path, 'w') as z:
        for name, data in entries:
            z.writestr(zipfile.ZipInfo(name), data)
    run(label + '-recover', path, recover)
    run(label + '-view', path, ['class-view', '--class-name', 'NestedEval'])

bad = bytearray(fixture)
assert bad[311] == 0x1a
bad[311] = 0xff
path = out / 'bad-body.class'
path.write_bytes(bad)
run('bad-body', path, ['class-view', '--class-name', 'NestedEval',
                      '--body', 'nestedPlain(I)I', '--body', 'nestedLocal(I)I'])
print('raw evidence:', out)
PY
```

## 门禁与完成判定

本轮在上述固定 HEAD 的代码上执行（后续工作树只改规划/文档）：

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --workspace --all-targets --all-features --locked` | seeds `5350648285461741569` / `5350648285461741570` 各 **1249 passed / 0 failed / 6 ignored** |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed（本机 JDK 23） |
| `cargo test --test p5_benchmark --locked -- --ignored --exact p5_repeated_direct_baseline` | 1 passed；18 filtered out，无性能阈值断言 |
| `cargo test --test p5_container_lookup --locked -- --ignored --exact container_lookup_timings` | 1 passed；29 filtered out，不产生本专项收益结论 |
| 修改前 OpenSpec strict | active + specs 19/19、archived 16/16 通过 |
| 修改后 OpenSpec strict | active + specs **20/20**、archived **16/16** 通过 |
| 本轮文档校验 | 17 份 Markdown 的 127 个本地相对链接无断链；修正 delta 保留主规格原有 12 个 scenario，新增 7 个；`git diff --check` 通过 |

6 个 ignored 分别是 JDK25 oracle 1、P3 编译对照 2、P5 基线 1、container 计时 1、fixture 指纹再生成 1；其中 P3 和两项 P5 已按上表另行显式运行。未运行会重写 fixture 的再生成，也未在本轮重跑 JDK25 oracle、MSRV、supply-chain 或 fuzz CI。门禁日志在本机 `/tmp/jarde-review-validation-*`；以上摘要和最小反例保存在仓库，临时日志不是永久验证资产。文中的最小复现脚本已从该 Markdown 提取并实跑，得到 R1/R2 表中结果。

上面两条反例在当前代码仍成立，常规门禁全绿不关闭它们。

完成条件：修正 change 的五项任务在固定提交验证通过，R1 不再伪唯一/伪缺失，R2 的库/CLI 状态一致，相关 delta 同步并归档。之后才更新 benchmark 的当前候选；`85828c4` 保留为历史比较臂。

## 修正结果（补记，2026-09-20）

上述两条反例已由 [preserve-task-operation-stops](changes/archive/2026-09-20-preserve-task-operation-stops/tasks.md)（5/5，固定提交 `8586356`）关闭，完成条件逐条满足。复核脚本原样重跑（库层值）：

| 场景 | 复核时 | 修正后 |
| --- | --- | --- |
| 有效 fixture | `Performed`/`complete`，exit 0 | 不变（正向对照） |
| 有效 → 损坏同名候选 | `Performed`/`complete`，执行了 body，exit 0 | `Incomplete`/`failed{classfile_decode}`，candidates=1，`method_bodies=0`/IR 0，exit 4 |
| 损坏 → 有效 | `Err(operation_target_not_found)`，exit 2 | `Incomplete`/`failed`，candidates=0，exit 4 |
| 一个 body 在 BCI 0 失败（另有正常方法） | 顶层 `complete`，CLI 靠自身补算才 exit 4 | 顶层 `partial{classfile_instruction_decode}`，正常 body 与成员表 coverage 保留，exit 4 由库报告决定 |

实现要点、变异证据（M1–M4）与门禁见该 change 的归档验证记录。**本复核的原始证据与结论保持原样**，未改写为事后通过；benchmark 当前候选随之更新为 `8586356`。

## 规划校正与分开处理的债务

- O1 定向容器访问和 backing/目录保留已在 `b22ea04` 交付；缓存现为 entries + retained_bytes 双限，默认 off。尚缺专项的 W1–W5、阶段归因、独立样本与 O1–O8 最终处置，不能将 22 项自动勾完。
- 同次 driver 的类名/access flags（包括 ACC_INTERFACE）已交接；MethodParameters、InnerClasses 的完整恢复消费和完整类级源码仍是后续范围。
- class_view 已共享一次物化字节/成员列举，但 reader 的 body 解码仍会重新解析类；成员表损坏可能连带拒绝原先可读的 body。这项局部解析/隔离债务需独立设计，不混入停止传播修正。
- `references` 按符号拼写查询，尚无物理身份直接查引用入口；WAR/Boot 自动 layout policy、现代源码输出、批量/并行/持久索引均不因本次任务链归档而成为已实现。
- 支持矩阵、路线、benchmark-review 中过期状态与归档链接已随本轮校正；历史 measurements/archives 不改写为当前运行结果。
