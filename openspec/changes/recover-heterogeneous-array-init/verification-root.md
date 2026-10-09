# Root Verification — EM-18 Assignability Slice

本片9/9任务完成，代码已提交推送main `29dcd5e892696e9f6b5adbb657ffa0a7db576c27`，确切SHA的CI37926854875四job、48steps全部成功。EM-18整单元保持部分完成。

## Architecture Acceptance

初始化器继续先满足既有 fresh-array、唯一消费者、物理存储顺序和效果闭包；只补齐元素到准确分量的兼容事实。标量复用现有平台谓词，等秩引用数组提升现有标量关系；自有类/接口复用同一 Runtime-selected、有界且计费的 header walk。proof 同时绑定真实 `aastore` BCI 和完整 source/component 拼写，保留元素表达式，不使用 invocation overload proof、不插入 element cast，无新 pass/crate/service。

root 修正了初实现的 selected-loader 参数缺失，并要求 store 精确读取三个连续绝对 stack operand，而不是假设 Stack(0) 或从多余 reads 中截取三个。primitive descriptor 与合法内部类名 `I` 在 reader 边界分开。报告注释已对齐既有 walk：header 直接命名的 target 无需物理存在；中间 header 必须选中并完整读取。

## Complete Comparisons

| 输入集合 | 原程序 | JADX | Jarde 旧行为 | 当前 candidate |
|---|---:|---:|---:|---:|
| 18 条冻结基线腿（双 javac，含 fresh/frozen CT） | 18/18 | 源码参考版 18/18 | 0/18 | 8/18 |
| v3 factory 完整六类家族，两条 compiler 腿 | 2/2 | rename-flags none 2/2 | 未以新输入重测旧 CLI | 2/2 |
| v3 direct-new 完整六类控制，两条 compiler 腿 | 2/2 | rename-flags none 2/2 | 未以新输入重测旧 CLI | 0/2，明确拒绝/编译失败 |

每个成功必须同时满足完整生成 source 集、无拒绝正文、空编译 classpath/sourcepath、验证运行仅使用新编译目录、退出码与原程序一致、stdout/stderr 逐字节一致。不能以 exit 0 或方法片段计成功。当前 candidate 的原18腿中通过 fresh/frozen CT、六 wrapper、Number[][] 各两腿；其余 inline-new/BigDecimal 失败保留，不计非法 Java。

原基线与 root 校验入口为 [results/README-baseline.md](results/README-baseline.md)。冻结 CLI SHA `0ebf4e6189c02d1d84072c9aeacd380303e71e6885f95408e054e8f843154f79`；三个产品源码 SHA 见 [results/candidate-cli-v1.json](results/candidate-cli-v1.json)。原18腿最终回放为 `results/candidate-v1-replay-v2/manifest.json`；root 的全 hash/双流审计为 `candidate-v1-replay-v2-root-verification.json`。

v3 family 的 root 独立回放为 `results/candidate-v1-fixture-v3/manifest.json`，SHA `d3dd331861d79d32cc64b2fd2e2c3677606868d54767e157fa99e5db7f508b87`。`results/candidate-v1-fixture-v3-root-verification.json` 核对246项 hash、四个输入的六类完整集合、隔离 argv/原始双流、两条正例共50个 initializer store 的 source-map 来源和无新增 element cast，零问题。完整 factory 包括六 wrapper、CharSequence、Collection、Throwable、平台数组提升、自有直接/两跳继承、interface、自有数组提升及同型/null/Object，trace 保留求值次序。

## JADX Reference and Evidence Corrections

本地 JADX 源码 SHA `2fb1b16386941660fda07e9017285aec40fcb37f`。v3 的默认与显式 `--rename-flags none` 共8条 profile/compiler 腿，全部完整编译/运行；仅 none 的4条双流一致。默认4条因默认包重命名为 `defpackage`，改变 `getClass().getName()`，全部记为语义未通过。源码依据为 `RenameVisitor.java` 的 default-package alias 分支，不归一化输出、不改生成源码。

agent 原 manifest 的 `initializer_methods_accepted_count` 实际只是语法检测，共72项；root 已独立纠正为 syntax observations，不作为完整语义分母。原始 manifest/raw 日志不修改，独立术语勘误保留。旧文件清单错误地给自身填写 hash，root v1 发现1项漂移；v2 清单排除自身与历史清单，root 重新核对211文件零问题。完整213文件归档为 `evidence/jadx-fixture-v3-reference.tar.gz`，SHA `7d828c23041fddbb9b895bd45c0683e6b3c06c17dde016ea6f3da24fcb6124fc`；见 `results/jadx-fixture-v3-root-verification.json`、`jadx-fixture-v3-archive.json`、`README-jadx-v3.md`。

## Boundary Tests and Actual Failures

focused 产品测试11+4通过；集成最终四项通过。删除 `Mid.class` 的两条 compiler 腿仍保持完整 Main 与其他类，DerivedA 到 Base/interface/数组分量的缺链明确在 BCI 24 以 Fallback/ExplanationOnly 拒绝；独立平台与 DerivedB 方法保持恢复。direct-new 控制保留实际 `jre_new_shape` 和五 wrapper 的 `jre_new_interleaved_effect`，没有伪称赋值非法。

早期集成测试失败均保存：v1 的 Rust partial-move；v2 的旧日志路径和不正确 OperationOutcome 预期；v3/v4 的错误 coverage/member-stop 假设。root 核对实际 API：零输出预算返回 Performed 的 partial class 和空正文/空映射 stopped member；analysis 总用量减一停在 body 全部尝试后的组装层，structural coverage 可 CompleteWithinSchema，但整体 execution 必须 Partial(AnalysisSteps)、明确诊断、用量不超限；动态分析始终 NotRequested。提前取消返回 Incomplete/Cancelled。没有为错误测试改变产品语义。最终记录为 `results/integration-census-v2/focused-integration-v5.*`。

新24 class 带172 Code bodies、8 branch targets、无新增 handler/subroutine。reader 的第一次普查失败如实保存，实测更新为1035/4468/457/2701/8；第二次普查、显式指纹生成及验证、P5 benchmark 都通过，见 `results/corpus-gates-v1/index.json`。

第一次全仓seed实跑350 targets，3318 passed/1 failed/93 ignored；唯一失败是旧varargs测试要求合法V3.viaMixed继续拒绝Number[]。root复核完整既有V3.class：原/JADX/Jarde各使用完整类，在javac8/23对应JDK隔离重编、验证运行双流一致（Jarde/JADX四条对照均通过），额外两JDK输入不混入原18腿分母。证据为 `results/legacy-v3-root/manifest.json` 与 `legacy-v3-root-verification.json`。旧测试改为精确完整表达式，原midStatement/doubleUse拒绝断言原样保留；原失败日志留在 `results/gates-v1`，第二轮完整门禁另记 `gates-v2`。

## Remaining Acceptance

最终两 seed 各350 targets、3319 passed/0 failed/93 ignored；MSRV1.88、fmt、CI-exact Clippy、显式 ignored P3及完整双JDK构造器 gate、strict OpenSpec和diff check均exit0。root核验38份原始双流hash（含明确保留且不计通过的首次census失败），见 `results/local-gates-root-verification.json`。任务3.2已完成。两seed后触及20GiB停建线，清理5323个文件/18.3GiB后继续剩余门禁；最终再次清理并核对主仓/fuzz无target、冻结CLI hash不变，清理后可用42,043,613,184字节，见 `cargo-clean-mid-gates.json` 与 `cargo-clean-final.json`。代码已提交推送main，root核对确切SHA的CI四job全部success、48steps，包括两seed和JDK25 oracle；任务3.3已完成。原JSON为 `results/ci-code-sha-29dcd5e89-success.json`，SHA `71cac4537552fe4ab1d0b5710c24fd69fe4a0ef8f1580bf5a8b5cdee82817570`，实际run为 [CI37926854875](https://github.com/LordCasser/jarde/actions/runs/37926854875)。当前工作区的下一片修改没有因此获得验收。14辅助工作树重新确认 detached、干净、为main祖先且无target；只有main/origin/main，见 `results/worktree-audit-before-commit.json`。

下一个现有证明组合任务为 [compose-constructed-reference-array-elements](../compose-constructed-reference-array-elements/)。direct-new 构造结果与 array element 尚无共同原子证明；五 wrapper 的 primitive argument conversion 另有构造证明边界。两者不靠泛化 Allocate 白名单掩盖，不关闭 EM-18。
