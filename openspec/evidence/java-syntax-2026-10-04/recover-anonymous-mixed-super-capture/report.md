# recover-anonymous-mixed-super-capture 实现与验收记录

实现者：coder subagent（2026-10-04，worktree 基线 `996ac2b7`）。范围：tasks 1.1–3.2（3.3 留 root）。
主锚按 root 同日裁决（选项 B）为**直返形混合 fixture** `tests/fixtures/proved-java-structure/anonymous-super-mixed-direct/`；冻结的赋值初始化形 fixture `anonymous-super-args/AnonymousSuperArgs$1` 保持物理呈现，其承接切片已立为 `openspec/changes/recover-anonymous-local-decl-site/`。

## 落点与三道门（root 钉死的均核实，另发现并报告了第四道前置）

1. **无 `val$` 证明器**：`prove_anonymous_capture`（facade.rs）按字段描述符二选一（`b"D"` → double，其余 → family），`irs` 在分派点已在手。已新增第三分支（字段名 `val$` 前缀）→ `member_inner::prove_anonymous_val_capture`；`prove_family_capture`、`prove_anonymous_double_capture`、`anonymous_double_constructor_shape` 零改动。
2. **`field_count != 0` 门**（`project_class_source_anonymous_super`）：放宽为"至多一个符合 `val$` 捕获形状（synthetic+final+instance、名字前缀 `val$`）的字段"，形状只预筛，证明仍由 `prove_anonymous_val_capture` 全量闭合。
3. **父构造器 descriptor 恰等门**：放宽为"父 descriptor == child descriptor 去掉捕获参数后的形状"，该形状由参数角色划分 `member_inner::partition_anonymous_val_constructor` 自证（super invoke 描述符与转发参数逐位一致），投影侧以 `analyze_method_ir` 自取 ctor IR 复用同一实现。`MemberCaptureProof` **未扩展**（pub+Serialize+deny_unknown_fields 契约零改动）。
4. **第四道前置（root 未钉死，已停手报 root，裁决 B）**：冻结锚的分配点在 `main` 的局部声明初始化位置，站点扫描（`class_source_direct_return_new`）、根方法返回门（`()Lparent;`）与赋值左端不可命名的匿名类型名三处使路径不可达；直返形锚所需的最小放宽（"局部声明前奏 + 直返"）已按 root 批准的锚形实现，赋值初始化形 + LHS 声明类型重拼移交 `recover-anonymous-local-decl-site`。

## 参数角色划分判据（`partition_anonymous_val_constructor`）

构造器无异常表、全部指令可归约：每个物理参数恰一次加载，加载值恰被一个已证槽消费——`putfield`（值位，另一读为 `this`）→ capture 角色；直超类 `<init>` `invokespecial` 实参位（接收者位除外，按弹出序还原左到右实参位）→ super 角色。拒绝：参数无消费、双角色、同类两位置、第二次 `putfield`、未证 sink、未证指令、super 实参非"物理参数前缀按序"、super 描述符与转发参数不一致。long/double 按描述符占两槽。投影侧另证：分配点唯一且 verified、AST 实参与站点实参 BCI 逐位一致、捕获实参为 `ExprKind::Local`、该槽恰一次写（或从未写的参数）且加载值 def 即该写。

## 主锚可观察行为对照（`anonymous-super-mixed-direct`）

原 class `java -Xverify:all`（`fixture-original/original-run.log`）：

```text
capture|arg:super-label|arg:super-value|base:explicit:17
explicit:17
capture|arg:super-label|arg:super-value|base:explicit:17|captured
```

投影后完整源集（根 + `Base`，`$1` 保持可查询不入源集）`javac --release 8 -g:none` **退出 0**（`mixed-direct-fixed/`），运行输出 `jarde-run.log` **逐行一致**（diff 为空）。呈现（`mixed-direct-fixed/AnonymousSuperMixedDirect.java`）：

```java
return new Base((java.lang.String) text("super-label", "explicit"), number("super-value", 17)) {
    java.lang.String render() {
        AnonymousSuperMixedDirect.event(local0);
        return super.render();
    }
};
```

捕获实参隐藏、构造器/字段/写入隐藏、捕获读取以根局部 `local0` 重拼；pre-super 写入由 javac 自行重建——**该 fixture 不再依赖 ctor 重排**（`recover-ctor-reorder-dispatch-guard` 的过渡收敛在其上被取代）。

## 冻结基线重放（`anonymous-super-args/`，tasks 1.2）

三类 SHA-256 与 2026-09-27 既有登记逐一一致；当前（guard 合入后）呈现为 verbatim 序 `this.val$captured = arg3;` 先于 `super(arg1, arg2);`（`baseline/AnonymousSuperArgs_1.java` 第 10–11 行）；完整源集 `javac --release 8` 退出 1（`baseline/jarde-javac.log`，灵活构造器预览错误）；原 class 事件日志（`baseline/original-run.log`）。按裁决 B 该 fixture 保持物理文本，由 `recover-anonymous-local-decl-site` 承接。

## 负例（六个，全部 `java -Xverify:all` 退出 0 且 Jarde 保持物理文本）

派生与逐项记录见 `negative-derivations.py` 与 `negative-derivations/`（SUMMARY.txt、前后呈现、运行日志）：

| 输入 | 形态 | 拒绝 |
| --- | --- | --- |
| unconsumed-param（锚 child 捕获存储 nop 掉） | 参数无消费 | `anonymous_super_capture_unproved`（划分：参数无已证消费） |
| dual-role-param（`aload_1`→`aload_3`） | 同一参数两类角色 | 同上（双角色） |
| reordered-super-args（前两实参换序 + 描述符改写 `(ILjava/lang/String;)V`） | super 实参序≠物理序 | 同上（非前缀按序） |
| double-write（结尾 `return` 换成二次 `putfield`+`return`） | 捕获字段二次写入 | 同上（`putfield` 数≠1） |
| multi-site（`two-mixed-sites/` 根 CP：第二站点类常量改名指向第一 child） | 两分配点同一物理类 | 站点唯一性关闭（无站点，物理文本） |
| two-capture-fields（源级：两个捕获局部） | 多 `val$` 字段 | `anonymous_super_child_shape_unproved`（一字段形状） |

CI 侧同形负例在 `tests/class_source.rs`：`anonymous_superclass_refuses_unproved_mixed_parameter_roles`（四项补丁）与 `anonymous_superclass_refuses_multiple_sites_and_multiple_capture_fields`，正例锚测试 `proved_anonymous_superclass_projects_the_mixed_capture_shape`。

## 门禁数字（最终文件状态）

- `cargo test --workspace --tests --locked --no-fail-fast`：**296 targets / 2937 passed / 0 failed**。
- `cargo fmt --all -- --check`：通过。
- clippy：按 `.github/workflows/ci.yml` 46–76 行逐字生成（`cargo clippy --workspace --all-targets --all-features --locked --` + 29 项 `-A` + `-D warnings`）：退出 0。首跑曾报 `clippy::unnecessary_map_or` 一处（member_inner `map_or(true, …)`），改 `is_none_or` 后干净。
- `openspec validate --all --strict`：**270 项全部通过**（269 既有 + 本片新增 `recover-anonymous-local-decl-site`；本片 tasks 1.1–3.2 已勾、3.3 留 root）。
- corpus 双腿扫描（`corpus-two-leg-scan/`）：`996ac2b7` 二进制 vs 本片二进制，49 渲染/腿，**差异仅 1 个文件**——新锚 `anonymous-super-mixed-direct` 根类（物理 → 投影），其余全部逐字节一致（含同形但被既有门拒绝的 `anonymous-capture`、`anonymous-top-level`）。
- `git diff --check`：通过（提交前 whitespace 检查无输出）。
- 磁盘纪律：每轮构建前 `df -h /`；最低点 12Gi（触发清理）；报告前已 `cargo clean`。

## `MemberCaptureProof` 契约

**未扩展**（root 决策落地确认）。扩展的只有非序列化、`#[doc(hidden)]`、单一构造点的发射 DTO `ProvedCapturedParameterRead`（新增 `parameter_presented`，替换原硬编码 `Type::Double`，对 String 捕获为正确性必需；既有 double 路径传 `Some(Type::Double)` 行为逐字不变）。

## 遗留缺口（如实登记）

- **赋值初始化形**（`anonymous-super-args/AnonymousSuperArgs$1` 的实际形态）：保持物理文本（响亮失败），由 `recover-anonymous-local-decl-site` 承接（站点扫描扩展 + LHS 声明类型重拼）。
- **Non-Goals 维持拒绝**：`this$0` + 捕获 + super 实参三者并存、嵌套匿名类、跨类引用、非 `structured` 正文、捕获字段二次写入、多 `val$` 字段——全部仍拒绝（负例已冻结覆盖后两项）。
- 直返形锚的 `event(local0)` 依赖 `-g:none` 下的合成局部名 `local0`；带 LocalVariableTable 的输入会拼回源名（既有命名通道，无需本片处理）。
- 接口两路径对 `val$` 形 child 的拒绝发生在更深处（`anonymous_child_capture_kind_unsupported` 硬拒绝），拒绝理由从"形状不符"变为"种类不支持"——可观察输出不变。
