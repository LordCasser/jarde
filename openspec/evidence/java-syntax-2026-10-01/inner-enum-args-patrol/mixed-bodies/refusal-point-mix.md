# 拒绝点定位：组合形态（构造实参 + 常量专属体）在主线 `03552a2e` 的义务证明（任务 1.1）

固定 fixture [fam.jar](fam.jar)（SHA 见 [sha256.txt](sha256.txt)，未改动）。javap 冻结的组合形态事实（`javap -p -c p/Combo.class 'p/Combo$1.class' 'p/Combo$2.class'`）：

- **主 ctor**：`private p.Combo(int)` → 物理描述符 `(Ljava/lang/String;II)V`，体为 `aload_0; aload_1; iload_2; invokespecial java/lang/Enum.<init>(Ljava/lang/String;I)V; aload_0; iload_3; putfield code:I; return`（8 条 = 4 + 1 参×3 + 1，恰为 arbitrary-arguments 通道 `prove_arbitrary_user_constructor` 的形状）。
- **桥 ctor**：`p.Combo(java.lang.String,int,int,p.Combo$1)` → `(Ljava/lang/String;IILp/Combo$1;)V`，体为 `aload_0; aload_1; iload_2; iload_3; invokespecial <init>(Ljava/lang/String;II)V; return`（marker 参数是声明位，桥体不读它；两个子类的桥目标是同一个 `Combo$1` marker——结构事实绑定，非名字绑定）。
- **子类 ctor**：`p.Combo$1(String,int,int)` / `p.Combo$2(String,int,int)`，体为 `aload_0; aload_1; iload_2; iload_3; aconst_null; invokespecial Combo.<init>(Ljava/lang/String;IILp/Combo$1;)V; return`。
- **`<clinit>` 常量步**：带体常量 7 条（`new $N; dup; ldc name; iconst ordinal; <用户实参>; invokespecial $N.<init>(Ljava/lang/String;II)V; putstatic`），普通常量 `ID` 同宽 7 条但 `invokespecial` 直指主 ctor；随后 `$values(); putstatic $VALUES; return`。`$values()` 15 条（`iconst_3; anewarray; (dup, iconst_i, getstatic, aastore)×3; areturn`）。

## 三层拒绝点（按 p.Combo 实际触发顺序）

| 层 | 位置 | 拒绝点 | p.Combo 触发 |
| --- | --- | --- | --- |
| 1 关系解析 | `src/facade.rs::resolve_enum_constant_body_relations` | 二元数量门：`allocations.len() != 2` 与 `constants.len() != 2`（3 常量直接 `Ok(Vec::new())`） | **首个拒绝点**（数量先于形状） |
| 1 关系解析 | 同上 + `enum_constant_constructor_matches` | 描述符白名单 `BASE_CTOR \| STRING_CTOR`（`(Ljava/lang/String;II)V` 拒绝）；`descriptor_source_argument_count` 只编码 0/1 | 若数量门放过，此处拒绝 |
| 2 常量步骤校验 | `prove_enum_body_initializer_prefix` / `prove_enum_body_values_factory` | 步宽硬编码 6/7（按 count）、String 之外实参形状无证明路径、`$values` 指令数硬编码 11 | 同上 |
| 2 常量步骤校验 | `prove_enum_constructor_instructions` | 转发序列四固定 opcode 表（零参/String，±marker） | 子类/桥转发证明无 int 参形状 |
| 3 呈现合成 | `class_source.rs::prepare_enum_constant_body_source_projection` | `group.constants.len() != 2` 门；实参拼写只有 `string_argument` 单 ASCII String 一条路 | 组证明从未到达此层 |

主枚举 ordinary 通道先行拒绝（`prove_group`：`enum constructor delegation edge refused: the physical method <init>(Ljava/lang/String;I)V is absent`——双构造器委托链形状不符），随后关系解析层二元数量门使 `enum_constant_body_relations` 为空，组合形态整组逐字段降级（[combo.base.java](combo.base.java)）。

## 参数化落点（本 change 实现后全部闭合）

1. `descriptor_source_argument_count: u8` → `source_parameters: Vec<EnumUserParameter>`（`parse_arbitrary_ctor_descriptor` 逐常量解析，BASE_CTOR 为空尾）。
2. 数量门放宽为 `≥2`（逐常量循环、sidecar 步数 = N+2、`$values` 长度 = 4N+3、`iconst` 序数改 `int_constant_at`）；0 参（`demo.Op`/`demo.Mixed` 形）与单 String（DT-12 形）两固定切片钉死恰 2 常量，行为逐字不变。
3. 主 ctor 链：空尾走纯转发、STRING_CTOR 走既有单 String 证明、其余走 `prove_arbitrary_user_constructor`（复用，返回 super-call BCI + user tail）；桥/子类转发证明按参数种类构造期望序列（slot ≤3 `_n` 形、≥4 两字节 load、末尾 marker null）。
4. 常量步实参证明复用 `prove_user_source_argument`（int 族窄化/String/getstatic 限定名/null）。
5. 呈现：`NAME(` + `user_arguments_source_text` + `) {` + 体方法文本 + `}`；构造器文本复用 ordinary 通道的 `spell_user_tail_constructor_text`（`private Combo(int arg0) { this.code = arg0; }`），不写第三份拼写。
6. 连带定位并修复同层第二处名绑定：`enum_access_constructor_marker_owner` 只认零参桥描述符（`(Ljava/lang/String;I` 后直接 `L…;)V`），组合形态 `IILp/Combo$1;)V` 解析失败 → 桥计数不匹配 → 关系解析整组拒绝；修复为按 `EnumUserParameter::parse` 跳过用户参数后取末参 `L…;`（marker ≠ 枚举自身名的规则保留）。
7. 构造候选不再依赖扫描侧 `new@1` 计划的 `verified` 位（该计划无法建模 getstatic 等实参形状，[gs-mix 变体](variants-before-after-mix.md)实证）：扫描只作"`new` 全量清单"（`complete` 门保留），构造关系由同 run Code 逐步重证（头指令、首个 `invokespecial`、putstatic 相邻、常量步逐参证明）。
