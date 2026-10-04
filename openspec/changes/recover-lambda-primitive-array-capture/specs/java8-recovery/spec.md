## ADDED Requirements

### Requirement: lambda 对原生数组捕获的 SHALL 以有据类型参与一致性判据

当 lambda 站点（`invokedynamic`）捕获一个**原生数组**局部（`newarray` 产物，如 `int[] t`；描述符 `[I`..`[J` 十种之一）且站点描述符与实现描述符对该捕获**一致**陈述为数组类型时，系统 SHALL 以**有据可查的该数组类型**参与捕获三方一致性判据（帧/站点/实现），而不得以帧层对 `newarray` 的保守 `Unknown` 引用（呈现为 `Object`）将其判为不一致——从而该形恢复为源级 lambda 捕获（与引用类型捕获同构的内联呈现）。

三方一致性判据本身（`frame_type != site_type || site_type != implementation_type` 则拒，`jre_lambda_sam_types`）SHALL **逐字保留**：站点与实现真不一致、或捕获值类型无可据陈述时仍 SHALL 拒绝。若实现走帧层给名（方向 A），`RefType::Unknown` 的既有消费面 SHALL 逐一核对无行为依赖（记录存证据）；若走陈述类型（方向 B），类型陈述 SHALL 来自字节码陈述的事实（LVT 或 `newarray` 的 `atype` 码），不得发明。

`multianewarray` 产物的捕获（`int[][]` 等）与 `anewarray` 路径（已恢复）不在本能力范围内。

#### Scenario: 原生数组捕获恢复

- **WHEN** 真 javac 8 与 javac 23 `--release 8` 双腿编译的 `P02_lambda`（`static int sum(List<Integer> l){ int[] t={0}; l.forEach(i -> t[0]+=i); return t[0]; }` 及 `map` 方法）经 `class-source` 呈现
- **THEN** `sum` 的 lambda 站点恢复为源级捕获内联（与 `map` 的 `StringBuilder` 捕获同构），源码区 0 引注（修复前 3 条：BCIs 15/8/9）；渲染源集 `javac --release 8` exit 0，运行输出 `6` 与原 class 一致

#### Scenario: 引用捕获与既有验收零回退

- **WHEN** `P02_lambda.map`（StringBuilder 捕获）与 DT-26 既有验收 fixture（`dt26-lambda-capture`）经呈现
- **THEN** 渲染文本与修改前逐字节相同

#### Scenario: 真不一致仍拒绝

- **WHEN** 合成探针使站点描述符与实现描述符对同一捕获操作数陈述不同类型（无据可查为一致）
- **THEN** 保持 `jre_lambda_sam_types` 拒绝——判据逐字未放宽，改变的只是原生数组的类型取值来源不再天然 `Object`

#### Scenario: 无版本判据

- **WHEN** 审查本片生产 diff
- **THEN** 无 `java_release`/`major_version` 分支；双腿（真 javac 8 / javac 23 `--release 8`）行为相同（本形本就双腿一致拒绝）
