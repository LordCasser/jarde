# `partialBreak` 的物理分支负例方案（未执行）

本方案只基于 change 内已冻结的完整原始 class、`javap -c -p -s` raw 和 public IR raw 做字节级规划。没有改动仓库，也没有运行 Git、Cargo、JDK、JADX 或 Jarde CLI。下面的 target/edge 都是拟议 classfile 变体的物理推导，**不是已经分析过的 canonical IR，也不是已验证可加载或可运行的 class**。

## 冻结输入与实际布局

目标方法为 `ConditionalSwitchBoundaries.partialBreak(II)Ljava/lang/String;`。两个原始输入来自 `results/boundary-baseline-root-v1/jarde-conditional-boundary-preflight-root-v2/cases/{javac8,javac23}/original/classes/ConditionalSwitchBoundaries.class`：

| class | SHA-256 | 长度 | Code 字节长度 | Code 首字节在 class 文件中的偏移 |
|---|---|---:|---:|---:|
| javac8 | `27dea02e6dc3003a822ef7db2f447a6d21a508d1fb83c83622f86c64f4ff1b8b` | 1494 | 79 | 817 |
| javac23 | `e51368a95639b9bba3cd94a0f93ac38c11e465b984644ca28c4dc88ab6d82eb5` | 1488 | 79 | 811 |

两份 class 的方法内 BCI 相同。原 `javap` 与 public-IR 指令事实：

- `lookupswitch@9` 的真实 case entries 是 `36`（case 0）、`57`（case 1）、`67`（default）；公共汇合块是 `74`。
- `ifeq@37` 原目标为 `50`，原指令字节 `99 00 0d`；相对偏移从 opcode 起算，`37 + 13 = 50`。
- `goto@47` 原目标为 `74`，原指令字节 `a7 00 1b`；`47 + 27 = 74`。
- 真实 canonical blocks 为 `0, 36, 40, 50, 57, 67, 74`，均 `path=[]`。
- 真实 canonical Normal rows：`0→36, 0→57, 0→67, 36→40, 36→50, 40→74, 50→57, 57→74, 67→74`。
- `36→40` 是 if 的物理顺序后继，`36→50` 是 ifeq target；`40→74` 是 `goto@47`。`50→57` 是 BCI56 `pop` 后的物理顺序后继。
- blocks 36/40/50/57/67 的入口状态可从真实 SSA 输出核对：local 0/1/2 存在、operand stack 为空。两个 proposed destinations 57/67 已经是原 switch dispatch 的真实入口，且已经有 StackMapTable 目标帧；这仅支持选择已有边界，不能替代 JVM 对变体的验证。

以上 facts 可在 `results/boundary-public-ir-root-v1/jarde-conditional-boundary-ir-root-v2/0.stdout.raw` 的 javac8/javac23 `partialBreak` sections 找到；原 javap 分别在 baseline preflight `guard-streams/4.stdout.raw` 和 `guard-streams/14.stdout.raw`。

## 两个独立变体

每个变体均从对应原始 class 的独立副本生成，只写 `Code` 中现存分支的两个 offset operand 字节；不改 opcode、Code 长度、其他指令、其他属性或 class 其他方法。每个 JDK 输入都应单独应用同一 BCI recipe，并独立核对变体 SHA。

### A：两个不同 case-entry exits

| 指令 | 原 target / 原 bytes | 新 target | 新 bytes |
|---|---|---:|---|
| `ifeq@37` | 50 / `99 00 0d` | 57 | `99 00 14` |
| `goto@47` | 74 / `a7 00 1b` | 67 | `a7 00 14` |

物理推导是：从 case 0 的条件 block 36，taken edge 到已存在的 case 1 entry 57；另一分支进入 block 40，再由现有 goto 到 default entry 67。因此，case 0 的可达路径有两个不同的 switch-entry exits，目标集合 `{57, 67}`。对应于原始 canonical 行的预期局部替换是 `36→50` 改为 `36→57`、`40→74` 改为 `40→67`；原 `0→57`、`0→67` dispatch rows 保留。BCI50 原块仍在 class 中，但因 ifeq 已不再指向它，它可能成为 unreachable；canonical observer 必须从变体真实图报告，不能把上述替换表当作完整 IR 结果。

### B：唯一但非相邻 case-entry exit

| 指令 | 原 target / 原 bytes | 新 target | 新 bytes |
|---|---|---:|---|
| `ifeq@37` | 50 / `99 00 0d` | 67 | `99 00 1e` |
| `goto@47` | 74 / `a7 00 1b` | 67 | `a7 00 14` |

从 case 0 的两个条件结果看，target 路径直接到 default entry 67；另一条路径先进入 block 40，再由 goto 到 67。因此可达 case-0 路径的唯一 case-entry exit 是 `{67}`。按 switch entry 次序 `36,57,67`，67 跳过中间的 case 1 entry 57，故此 exit 非相邻。它把“唯一出口但非相邻”与“多个不同出口”分开。原 block 50 仍存在但无此条件入边，需由真实 observer 报告 unreachable/ownership，不可伪称由case0到达。

## 变体验收前的必要核验

1. 仅复制冻结 class 到独占临时输入；按上表断言 Code BCI、opcode、原 target 和新 3-byte 序列。除指定 operand 的差分必须为空；class 级差分也应仅为对应两个 offset 字节。
2. 由 root 在受控 JVM 下先做 classfile/JVM verification。现有目标已经是 switch entries 且这些块的原 SSA 入口 stack 为空，局部状态同属此方法；但此静态观察不证明修改后 StackMapTable/验证状态一定可接受。
3. 用既有 public reader 重读变体的真实物理解码、canonical blocks、完整 canonical edge multiset、unreachable rows 和 SSA。确认 goto/conditional offset 被 decoder 解释为新目标，并确认目标仍是原有 case-entry physical BCI。不得预填 synthetic canonical rows。
4. 只有真实 IR 形状分别满足 A `{57,67}` 两个可达 exits、B 唯一可达 case exit `{67}` 且 entry 次序证实非相邻时，才把其作为对应 boundary test 输入。若 JVM 或 canonical 重读不支持，保留变体失败 raw 并停止采用该 recipe。

## 结论范围

当前静态事实足以给出两条具体、同方法、同两套冻结 class 的 offset-only recipe；它们不是新的源码 fixture，也没有运行接受结论。此方案只覆盖 `partialBreak` 的多出口与唯一非相邻入口边界，不扩展其它 method、case 语义或 proof 机制。
