# Design：写访问器字段类型泛化

## Context

真 javac 对"内部类写外部私有字段"生成返回值形 `access$NNN`（值经 `dup_x1`/`dup2_x1` 复制后既 `putfield` 又 `xreturn`）。现有处理器 `BooleanAccessorAssignment`（`build.rs:8755` 起）已验证该结构，但每处判据都硬编码 boolean：

- 描述符模板 `format!("(L{owner};Z)Z")`（build.rs:8799 附近）
- opcode 表 `[0x2a, 0x1b, 0x5a, 0xb5, 0xac]`（= `aload_0, iload_1, dup_x1, putfield, ireturn`）
- BCI 表 `[0, 1, 2, 3, 6]`
- `code.max_stack < 3 || code.max_locals < 2` 下限
- `field.descriptor != "Z"` 即拒（build.rs:8848-8851）

**相邻但不相干的姊妹路径（勿动、勿混）**

> **root 勘误（2026-10-05，实现者发现）**：上方"消费链零改动"的预审计结论**漏记了一道门**——`assignment_result_statement`（build.rs:22538 附近）持有 `evidence.descriptor != if boolean_accessor {"Z"} else {"J"}` 的描述符检查（两调用点 18390/18401），非 Z/J 描述符必被它拒成空 stub。该门的 `"Z"`/`"J"` 常量与 prove 侧 `field.descriptor != "Z"` 是**同一类型事实的两个副本**，泛化它们属决策 1 字面范围（root 已裁决，附带条件：J 臂改后渲染逐字节实证、本勘误留档、零回退锚不放松）。：`LongAssignmentResult::prove`（`build.rs:8559`）也认 `dup2_x1`（`OPCODE_LLOAD_1=0x1f`/`OPCODE_DUP2_X1=0x5d`/`OPCODE_LRETURN=0xad`，8577-8582），但其判据 `has_receiver && parameters == 3`（8606-8607）表明它服务的是**实例方法**的 `this.f = v; return v;` 赋值表达式（普通方法体内的 `(o.f = v)` 消费形），**不是** static `access$NNN`。实测旁证：`WA` 的 static `access$202`（long）被拒（"BCI 2 not part of the provable subset"），因为 static 形 `has_receiver=false` 直接不匹配该路径；而 boolean 的 static `access$002` 由 `BooleanAccessorAssignment` 恢复——该 wrapper 在 8915 附近**手工构造** `LongAssignmentResult`（绕过 prove 的实例门）。**即 static 写访问器路径只有 boolean wrapper 一条**，本片泛化的就是它；`LongAssignmentResult::prove` 的实例形判据**逐字不动**。

## 决策 1：按描述符查的每型事实表（封闭集合）

把硬编码 boolean 改为一张**封闭的**每型表，键 = 字段描述符（即访问器参数/返回描述符），值 =（装载 opcode、返回 opcode、复制 opcode、槽宽）。表内容以巡查实测为准（[results3/nine-type-measurement.txt](../../evidence/java-syntax-2026-10-04/value-returning-write-accessor-patrol/results3/nine-type-measurement.txt)，javap 原始记录在 results3/）：

| 描述符 | load@1 | copy@2 | return@6 | 槽宽 | stack/locals 下限 |
| --- | --- | --- | --- | --- | --- |
| `Z` `I` `B` `S` `C` | `0x1b` iload_1 | `0x5a` dup_x1 | `0xac` ireturn | 1 | 3/2 |
| `F` | `0x23` fload_1 | `0x5a` | `0xae` freturn | 1 | 3/2 |
| `L…;` `[…` | `0x2b` aload_1 | `0x5a` | `0xb0` areturn | 1 | 3/2 |
| `J` | `0x1f` lload_1 | **`0x5d` dup2_x1** | `0xad` lreturn | 2 | 5/3 |
| `D` | `0x27` dload_1 | **`0x5d`** | `0xaf` dreturn | 2 | 5/3 |

（每型十六进制已与**仓库既有常量**交叉核对：`build.rs:8579-8582` 的 `OPCODE_LLOAD_1=0x1f`、`OPCODE_DUP2_X1=0x5d`、`OPCODE_LRETURN=0xad` 与本表一致——root 初稿曾把 `lload_1`/`fload_1`/`dload_1`/`dup2_x1` 写成各自**相邻** opcode（0x1e/0x22/0x26/0x5b，分别是 `lload_0`/`fload_0`/`dload_0`/`dup_x2`），靠仓库常量才纠正；实现片不得凭记忆写十六进制，一律引用仓库常量或 javap 实测。）

描述符表**不在表内**（如嵌套数组、其它原生型不存在于 Java 字段）→ 维持现状拒绝。**封集是设计约束**：新类型必须先取证（javap 实录）再入表，不允许实现片自行外推。

## 决策 2：所有非类型判据逐字保留

处理器的**结构判据**与字段类型无关，全部不动：单 SSA 块、无异常表、无 clone 块、BCI 布局 `[0,1,2,3,6]`、`Operation` 序列（`Load{slot:0}`/`Load{slot:1}`/`Other`/`Field{Write, !static}`/`Return`）、`fields.claim(3)` 所有权、`field.owner == owner`、预算 charge。**这正是"泛化一个已验证处理器"与"新机制"的区别**：结构证明零改动，只把"类型事实"从常量变为查表。

## 决策 3：呈现与命名

- 呈现保持 boolean 现状：`arg0.<field> = arg1; return arg1;`——所有类型的语义相同（返回写入的值）。
- 处理器命名：倾向把 `BooleanAccessorAssignment` 改名为 `WriteAccessorAssignment`（或 `AccessorFieldWriteAssignment`），boolean 成为表中一行而非特例。改名涉及调用点（`build.rs:7293`）与结构体字段（`build.rs:8221`），机械同步即可。若实现中发现改名波及测试断言文本，允许保留旧名 + 注释说明，但**不得**为避免改名而复制一份平行处理器（那会制造两条可漂移的判据）。

## 决策 4：零回退锚

- `d09f5dea` 的 boolean fixture（`openspec/evidence/java-syntax-2026-09-27/dt29-reference-cast-audit/fixtures/private-field/`）渲染**逐字节不变**。
- 现有 `tests/` 中行使 `BooleanAccessorAssignment` 的断言（含 `p3_accessor_edges` 家族）不因改名/泛化而变红；若断言文本引用了结构体名，随改名机械更新。

## 验证标准（可证伪）

1. **主锚**：冻结的 `WA` 族（真 javac 8，[fixture/WA](../../evidence/java-syntax-2026-10-04/value-returning-write-accessor-patrol/fixture/WA/)）渲染：9 个写访问器**全部恢复**为 `arg0.x = arg1; return arg1;` 形，源码区对这 9 个方法 0 引注；渲染源集 `javac --release 8` **exit 0**（修复前 8 个 "缺少返回语句"），运行输出与原 class 逐行一致（`main` 打印 9 字段拼接串）。
2. **零回退**：boolean 先例逐字节不变；`accessor.rs` 路径的既有负例（`returns.is_some()` 拒绝形，如 `p3_accessor_edges` 的无复制返回形）仍拒。
3. **负例**：(a) 描述符不在表内（若可构造——预期 Java 字段类型都在表内，则用合成 classfile 探针）→ 拒；(b) static 字段形 → 拒（现有判据）；(c) 破坏结构判据之一（如两块、有异常表、BCI 偏移）→ 仍拒（结构判据零放宽）。
4. **corpus**：双腿（基线/修复）全语料渲染 diff——只应出现**真实含写访问器的类**的差异；语料普查已知 0 个真实 `access$` 产物，故预期 diff 为空（这本身就是回归证据）；出现任何其它差异即停下报告。
5. **门禁**：全量测试（基线 301 目标/2970 passed；flake 家族单测复跑两轮判定）、fmt、CI-exact clippy、openspec strict、`git diff --check`、fixture 新增后再生 corpus fingerprint。

## Open Questions（root 2026-10-05 派发前预审计收窄）

1. ~~`LongAssignmentResult` 对双槽 `dup2_x1` 是否需要区分槽宽~~ **已收窄**：root 读码 `assignment_result_statement`（build.rs:22517 起，消费方）与 `field_value`（build.rs:22644 起）——消费链是**按 SSA 值**（`receiver_source`/`parameter_source`/`field_copy` 的 `render_value`）走、槽宽语义由 `field_value` 的**描述符驱动**分支（`descriptor_type` → `Type::Boolean` 特判 / `Byte|Char|Short` 收窄 cast / 其余 `meeting_position`）承担，`LongAssignmentResult` 自身**不携带也不需要槽宽字段**。即双槽形在既有结构内可承载，实现片泛化时只需让 prove 接受 `J/D` 的 load/copy/return（查表），消费链零改动。但注意 `field_value` 的 boolean 特判读 `evidence.descriptor`——泛化 boolean 之外各型时确认该分支不误触发（`Type::Boolean` 判断已天然只对 `Z` 成立，预期无冲突，实现时以测试钉死）。
2. 引用型描述符的表键匹配：~~预期描述符本身就是访问器参数描述符~~ **已确认为常规事实**：`fields.claim(store_bci)` 返回的 `evidence.descriptor` 即字段描述符，与访问器参数描述符一致是 `BooleanAccessorAssignment` 既有判据（`field.descriptor != "Z"` 那条）的同一事实源——泛化时保持"`field.descriptor` == 表键 == 参数/返回描述符"三方一致即可，无需前缀解析。

1. `LongAssignmentResult`（处理器内部承载的结构）对双槽 `dup2_x1` 是否需要区分槽宽——预期不需要（它承载赋值结果，槽宽由描述符事实决定），实现片在泛化 long/double 时验证；若需要，在既有结构内加槽宽字段，不新建平行结构。
2. 引用型描述符的表键匹配：`L…;` 与 `[…` 前缀如何与"字段描述符 == 访问器参数描述符"一致性判据协作（预期描述符本身就是访问器参数描述符，直接 `field.descriptor == 参数描述符` 即可，无需前缀解析）——实现片确认。
