# 非静态成员类折叠取证（task 1.1/1.2，change `recover-inner-class-instance-folding`）

基线：worktree 主线 `c7b7b767`（实现前重放，debug 构建）。N1 fixture SHA 与
[`../results/fixture-sha256.txt`](../results/fixture-sha256.txt) 逐项核对一致；原 jar
`java -Xverify:all` 运行 `10`/`7`/`13`（`../fixture/orig.out`）。

## 1. 三项取证（design Context）

### (a) javap N1 家族——ctor 首参/this$0 序、access 桥、限定 new 降低

`N1$Inner.<init>(LN1;I)V`（javac `--release 8`）：

```
0: aload_0          // this
1: aload_1          // 首参 = N1（synthetic this$0 位）
2: putfield this$0:LN1;     // this$0 写在 super() 之前
5: aload_0
6: invokespecial java/lang/Object."<init>":()V
9: aload_0 / 10: iload_2 / 11: putfield tag:I / 14: return
```

即 ctor 首参即 this$0 位、捕获写在 super 前（synthetic-ctor 片已证序）；**普通用户字段写
（tag）与 this$0 写同体共存**——六指令模板证书（`capture_constructor_shape`）只覆盖无用户
工作的 ctor，需要一般化（见 §3 提取落点）。

`access$000(LN1;)I`（`N1` 的 ACC_SYNTHETIC 静态桥）：体恰为
`aload_0; getfield base:I; ireturn`（3 条指令、无异常表）。`total()` 里的调用形：
`getfield this$0 → invokestatic N1.access$000` —— 读桥调用位可重写为
`<this$0 读值>.base`。

限定 new 的两种降低（`N1` 的 `main` 与 `N1$Stat.use`）：

- **静态语境 `new N1().new Inner(3)`**（main，BCI 12-24）：
  `new N1$Inner; dup; new N1; dup; invokespecial N1.<init>; iconst_3; invokespecial
  N1$Inner."<init>":(LN1;I)V` —— **限定符是新分配**（无空检查）：fresh-allocation 形。
- **参数限定符 `outer.new Inner(9)`**（`Stat.use`，BCI 0-12）：
  `new N1$Inner; dup; aload_1; dup; invokestatic Objects.requireNonNull:(Object)Object; pop;
  bipush 9; invokespecial …` —— requireNonNull 舞蹈形（既有 `verify_member` 舞蹈臂已证）。

### (b) mixed 装配缝的非静态候选选择点

`scan_family_root` 的非静态候选从不进 static 行收集（`StaticMembersWithInstance {
statics, candidate }` 之后 facade 在缝上分解：`candidate` 走 `Candidate` 同一窄通道分支，
`statics` 走静态折叠通道）。本片选择点即该缝：窄通道 `Prepared` 且投影未 Projected 的非静态
候选（`relation.access_flags & ACC_STATIC == 0`）加上已装配的静态子集，进入**联合实例折叠**
（`prepare_class_source_instance_member_fold` → `project_class_source_member_fold`）；联合失败时
——混合族回退静态子集折叠（上片合同），纯非静态族不发布任何折叠（窄通道家族原样保留）。

### (c) 限定 new 恢复的真实拒绝码

分离呈现下 `Stat.use` 的拒绝：恢复层无 member target（`new@1` 对 `new N1$Inner` 拒绝：
"an allocation, a copy or a cast is presented only where a rule proved what it builds"）——
**不是形状不可证，是 target 缺席**。折叠缝补上 `ProvedMemberInnerTarget` 后，同一
`verify_member` 的舞蹈臂直接证明（`jre_new_member_shape/order` 家族拒绝码不变）。
结论：**舞蹈证明不是前置缺口**；三种限定语境（外围 this / 参数+舞蹈 / 新分配）全部落形：

| 语境 | 降低 | 证明臂 | 呈现 |
| --- | --- | --- | --- |
| `make` 内 `new Inner(t)` | `aload_0` 直通（this 无需空检查） | 新增 enclosing-`this` 臂 | `new Inner(args)` |
| `outer.new Inner(9)` | `aload_1; dup; requireNonNull; pop` | 既有舞蹈臂 | `arg1.new Inner(9)` |
| `new N1().new Inner(3)` | 嵌套构造完成值直通 | 新增 fresh-allocation 臂 | `new N1().new Inner(3)` |

**受阻面（如实登记，不放宽）**：孙代链 `a.new B().new C()`（IV3）——B 的 member 呈现类型
是源拼写 `IV3.B`，而 C 的 ctor 描述符要求池拼写 `IV3$B`，参数引用转换无证据（"no safe
reference conversion evidence"）；折叠按"重跑不得降级"门保守拒绝（见 README §3）。

## 2. 变体冻结（task 1.2，`fixture-variants-inst/`，`javac --release 8 -g:none`）

| 变体 | 形态 | 原 `-Xverify:all` 输出 | 实现前 member_family |
| --- | --- | --- | --- |
| IV1 | 双私有捕获字段（base+bonus）→ 双读桥 access$000/access$100 | `12` | prepared（Inner），投影 refused |
| IV2 | 复合写 `IV2.this.count = IV2.this.count + 1` → 读桥+写桥（access$002 返回新值） | `11` | prepared，投影 refused |
| IV3 | 孙代链 `a.new B().new C()`（B 直接成员、C 孙代） | `6` | prepared（B），投影 refused；main 的链构造本就未恢复 |
| IV4 | 源码声明的 `static int access$000(IV4)`（**非** ACC_SYNTHETIC） | `104` | prepared，投影 refused |
| N1 | 巡查固定 fixture（三项语境 + 读桥 + 静态 Stat 混合） | `10/7/13` | prepared，静态子集已折（上片） |
| MV1–MV3 | 上片混合变体（纯非静态 / 单静态+单非静态 / 双静态+单非静态） | `9`、`11/8`、`6/6` | 静态子集已折，Inner 分离 |

实现前后逐类文本见 [`before/`](before/)（`*-inst-before.txt`，主线 `c7b7b767`）与
[`results/`](results/)（`*-inst-after.jarde.java`）；child 分离单元全部逐字节一致（负例 V5：
分离呈现零回退）。class SHA 见 [`results/sha256-inst.txt`](results/sha256-inst.txt)。

## 3. 提取落点（design 决策 1–3 的落点）

- **折叠机械**：静态折叠四件套原样复用（`nested_static_member_source_text` 加
  `elision: Option<InstanceMemberElision>` 参数；`source_text_with_nested_declarations` 加
  `hidden: &[HiddenFoldMethod]`）。族形态新增 `ClassSourceMemberFamily::PreparedFold`。
- **一般化捕获证书**：`prove_family_instance_capture`（member_inner.rs）——ctor 可含用户
  工作；putfield 位置任意；SSA 证明写读 this+首参 entry 值、首参单用、无异常表、无
  method-handle。六指令模板证书保留给窄通道。
- **消隐**：ctor 首参消参（`top_level_parameter_end` 定位首参 token）+ 捕获字段行隐藏 +
  捕获写语句删除（`fold_capture_write_statement_span`：primary origin = putfield 的单行
  segment）+ 读桥隐藏与调用位重写（`fold_bridge_call_rewrite_span`：`Owner.access$NNN(arg)`
  segment → `arg.field`；写桥/非 ACC_SYNTHETIC/非直通体不隐藏）。
- **限定形**：`verify_member` 三个臂（§1c 表）；重跑经 census 只对受影响家族方法注入
  member targets + captured outer reads。
