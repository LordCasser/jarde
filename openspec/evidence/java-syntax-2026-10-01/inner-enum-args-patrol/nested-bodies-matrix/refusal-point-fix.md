# 嵌套枚举常量体折叠：首个拒绝点定位与名派生修复记录（`recover-nested-enum-constant-bodies` 任务 1.1/2.1）

2026-10-01，worktree 基线 `ed41efe6`。判别变量复认：`p.Holder2$OpAbs` / `N2$Operation`（枚举二进制名含 `$`）逐字段降级，`demo.Op` / `p.OpIface`（顶层名）已折叠；与抽象/接口实现无关。

## 1. 排除过程（逐门取证）

冻结 jar `nested-iface-abs.jar`，CLI `jarde-cli class-source --input nested-iface-abs.jar --class 'p.Holder2$OpAbs'`（修复前输出存档为本目录 `p.Holder2_OpAbs.jarde.java`：`ADD;`/`MUL;` 逐字段）。常量体折叠链路为 `src/facade.rs` 的 `resolve_enum_constant_body_relations`（装配门 `may_capture_group_code` 不含名段切分），诊断时在该函数全部 31 个 `return Ok(Vec::new())` 早退点与 10 个 `continue` 点插入顺序标记，逐步跑冻结 jar：

- 调用门四条件（`capture_enum_group_code` / 普通证明 `Refused` / `structure_complete` / 执行 `Complete`）对嵌套枚举全部成立——`N2$Operation` 带 `ACC_ABSTRACT`（javac 对常量体枚举的语义标记），普通 `prove_group` 以 "the class is not a concrete Java 8 enum extending java/lang/Enum" 拒绝属预期触发，非本缺陷。
- 结构事实门（`<clinit>` Code 候选、分配扫描、两常量字段、构造点 owner、`$VALUES`、隐式成员、抽象/接口方法读取、子类读取、typed `InnerClasses` 自行、`EnclosingMethod` 指向枚举）**全部按结构事实通过**——现实现的子类归属不是字符串切分派生。
- 命中的唯一早退点：**构造器/访问桥整组门**（修复前源 `src/facade.rs` `resolve_enum_constant_body_relations` 内 `bridge_indexes` 校验，标记编号 #26）。拒绝时该函数返回空 relations → 无投影 → 逐字段呈现。修复前后该函数再无其它早退点被触发。

## 2. 确切拒绝点与名派生语义

被拒子句：

```rust
bridge_indexes.iter().any(|(table_index, owner)| {
    !constructors.iter().any(|c| c.table_index == *table_index && c.has_code)
        || !constant_shape.iter().any(|constant| {
            constant.allocation_owner == *owner        // ← 嵌套名在此为假
                && constant.allocation_owner != enum_owner
        })
})
```

即"synthetic 访问桥的 marker 参数类型 == 本组某常量的匿名子类名"。该等式只在**顶层枚举**的 javac 形态下成立（`demo.Op` 的桥为 `(Ljava/lang/String;ILdemo/Op$1;)V`，marker 即首个常量子类 `Op$1`）。嵌套枚举的 javac 形态不同：marker 取**最外层外围类**的合成匿名类——`javap` 冻结 jar 实测：

- `p.Holder2$OpAbs` 桥 = `(Ljava/lang/String;ILp/Holder2$1;)V`；`p/Holder2$1.class` 为 `ACC_SYNTHETIC` 空类（super `Object`、零成员，`EnclosingMethod → p.Holder2`，匿名 InnerClasses 行 `outer=0/inner_name=None`）。
- `N2$Operation` 桥 = `(Ljava/lang/String;ILN2$1;)V`，`N2$1` 同形。
- 常量的 allocation owner 是 `p/Holder2$OpAbs$1`、`p/Holder2$OpAbs$2`——永不等 marker，故整组门恒拒。

语义结论：marker 参数类型是 javac 的**重载消歧产物**，不是"哪个常量使用此桥"的证据；按 marker 名绑定桥→常量属于顶层形态专用的名派生假设（并非 `rfind('$')` 式段切分，而是同类的名形假设）。

## 3. 修复（结构事实绑定）

`src/facade.rs`（唯一功能改动点）：删除该名绑定子句，改为 marker ≠ 枚举自身名的单一名字规则；桥与常量的绑定完全交给既有**结构证明边**，不放宽任一义务：

- 桥身份：同组唯一 synthetic 构造器、有 Code、无未解释构造器（门内保留的计数/Code 检查不变）。
- 桥→私有构造器：`constructor_chain` 逐指令证明桥体原样转发 name/ordinal（`prove_enum_constructor_instructions`）。
- 子类→桥：每个带体常量的子类构造器由 `prove_enum_physical_constructor` 证明恰以该桥描述符调用且 marker 传 `null`（`aconst_null`）。
- 独占使用普查、ctor 委托体逐指令、纯覆盖成员集、structured 体方法、整组原子性全部沿用既有通道，零放宽。

回归证明：新增 5 项 in-crate 测试（`src/facade.rs` `enum_constant_body_relation_tests`），其中嵌套两格测试在**修复前代码上失败**（还原旧子句复跑验证：`nested_enum_constant_bodies_fold_with_the_same_structural_obligations`、`frozen_nested_matrix_jars_flip_exactly_the_nested_cells` 均以 `Refused` 失败），修复后通过。

## 4. 四格矩阵前后（CLI 文本 diff 断言）

| 格 | 修复前 | 修复后 | 断言 |
| --- | --- | --- | --- |
| `demo.Op`（顶层抽象） | 折叠（冻结 `demo_Op.jarde.java`） | 折叠，`demo_Op-fix.jarde.java` **与冻结文件逐字节相同** | 不变 |
| `p.OpIface`（顶层接口） | 折叠（冻结 `p.OpIface.jarde.java`） | 折叠，`p.OpIface-fix.jarde.java` **与冻结文件逐字节相同** | 不变 |
| `p.Holder2$OpAbs`（嵌套抽象） | 逐字段（冻结 `p.Holder2_OpAbs.jarde.java`） | 全量折叠（`p.Holder2_OpAbs-fix.jarde.java`：`ADD { public int apply … }`、`MUL { public int apply … }`） | 翻转 |
| `N2$Operation`（嵌套接口） | 逐字段 | 全量折叠（`N2_Operation-fix.jarde.java`：`PLUS { … }`、`MINUS { … }`） | 翻转 |

变体三例（负例/边界）见 [variants-before-after-fix.md](variants-before-after-fix.md)；三方行为对照见 [threeway-fix.md](threeway-fix.md)；产物 SHA 见 [sha256.txt](sha256.txt)。
