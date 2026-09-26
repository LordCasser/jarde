# 2.2b 独立验收：选定输入的子类 owner 使用清点

每个已选子类关系现在对所选输入的物理范围运行现有 P1 `Owner` 候选扫描，私有侧车保留每份输入的 XRef 坐标、扫描范围、覆盖率、停止状态和诊断。只有扫描完整、无未知或不支持类别，且每条精确引用均能归入同一 `<clinit>` 的 `new`/`invokespecial` 构造点或已核验的结构元数据时，才将该关系的“使用独占”标为真。主类的唯一 synthetic 访问构造器描述符、主类/子类自身及同组选定兄弟类的唯一匿名 `InnerClasses` 行是结构引用；带前缀或嵌套容器根没有对应的精确 P1 范围时保守拒绝。额外 Code 引用和篡改兄弟类行的控制均拒绝；同组另一个常量重用该 owner 时两条关系均不独占。紧预算停止不等同于零引用。

代理在冻结的 `Op`、`Mixed`、`Plain` 的 Java 8 `-g`/`-g:none` 输入上验证正例、拒绝控制和 Stage/Measure。Root 审核了完整白名单及扫描覆盖门，独立复跑 `enum_constant_body_relation_tests` 2/2、`enum_constant` 30/30、`class_source` 47/47、`jarde-query` 的 `owner_candidates` 2/2；`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-enum-constant-bodies --strict` 通过。首次复用 Root 私有 Cargo target 时拿到旧 `jarde-query` 产物并报缺 `Owner` 变体；清理该包缓存后重编及上述测试通过。代理全库 `jarde --lib` 61/61 与 Clippy 成功，Clippy 仍有既存告警。

本步仅证明所选范围内没有未解释的 owner 引用；还没有核对 `<clinit>` 的 name/ordinal 实参和对象别名、构造桥的参数转发、子类覆写方法或整组投影。基础枚举组对带体 `Op` 仍为 `Refused`，没有提前发布 `Proved` 或 Java 常量类体。已知 P1 golden 基线失败见 [2.1c 验收](verification-2.1c.md)，与本步无关。
