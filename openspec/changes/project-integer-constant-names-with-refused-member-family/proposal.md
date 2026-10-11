## Why

现有同类整数常量名称投影已经证明字段候选来自本物理类的完整 `ConstantValue` 表，使用同次方法 AST，检查名字遮蔽和同值歧义，并以物理字段、方法 BCI 与确切源码范围记录结果。但类级入口只在 `member_family == Absent` 时调用该投影，因此 `scan_family_root` 因 root self row 保守拒绝时，即使没有保留 child report、没有 nested family 文本，仍会漏掉独立的 root 方法常量名称。

冻结的上游 `TestSwitchLabels$TestCls` 是实际反例。`InnerClasses` 同时记录 root `TestCls` 属于 `TestSwitchLabels`，以及 direct child `Inner` 属于 `TestCls`。root family scanner 在 self row 上返回 `Refused { child: None }`，使当前 Jarde 源码把 `case CONST_ABC` / `return CONST_CDE` 写成 `case 2748` / `return 3294`；原源码和 JADX 都使用同类常量名。该差距与 switch 控制流恢复无关。独立选中的 `Inner` 也被同一个 RefusedNone 门挡住，其本类私有常量 3294 应可命名；只有外层字段存在的 2748 必须继续写数字。

## What Changes

- root report 执行为 Complete、三个 sibling family 均为 `Absent` 时，允许现有整数常量投影在 member family 为 `Absent` 或 `Refused { child: None }` 的状态下运行。
- 保留既有字段候选、同次 AST、遮蔽/重复值、全文 writer 重建、物理 anchors、精确 spans、预算与原子发布约束。
- `Prepared`、`PreparedPair`、`PreparedStatic`、`PreparedFold`、`Refused { child: Some(..) }`、`RefusedPair` 和任何非 `Absent` sibling family 继续阻止投影。
- 不解析 refusal reason、不改 family scanner、不装配 child 文本、不绑定 outer 常量；后者作为独立债务保留。
- 用真实冻结 class 验证目标行为，并在现有完整源对照中比较原源码、JADX 与 Jarde。验收范围是本窄 gate，不代表整 CF12 完成。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：同类整数常量投影不受无 child report 的成员 family refusal 阻止。

## Impact

只修改 `src/facade.rs` 现有 `project_class_source_integer_constant_names` 调用 gate。新增永久测试放在现有 `integer_constant_name_tests` 模块，复用 frozen `TestSwitchLabels$TestCls.class`、reader/class-source 公共 API 与已有物理 anchor 类型。验证使用现有仓库 harness 与冻结 CF12 输入；不增加产品机制、公开 schema、依赖或 reader/API。

前置条件：conditional switch 产品6476b56c357443ef17318891b12142f509977234必须先完成自身CI及干净主线关闭；本方案不表示补丁或新测试已经应用、编译、验收。外层字段绑定仍是独立后续债务，单class输入没有可验证的外层字段字节。
