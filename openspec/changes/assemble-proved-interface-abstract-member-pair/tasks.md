## 1. 固定两成员目标

- [ ] 1.1 从 EM-01 `multi` 分离 `Shape` 完整 Java 8 类与外部 Runner；核 pinned JADX HEAD、三个 class SHA、双方 `InnerClasses`/flags/方法表及原/JADX 源码重编、`-Xverify:all` 和反射结果。
- [ ] 1.2 冻结 `Shape` 根与两个 child 的当前 Jarde 物理报告、缺失声明与首个拒绝门；制作至少缺 child、错误 self row、第三 child、接口方法有 Code/错误 flags、额外字段或 Signature 的可解析近邻。

## 2. 同轮联合家族证明

- [ ] 2.1 在现有 `scan_family_root`/`child_relation_agrees` 路径识别恰好一个接口加一个抽象类的直接声明组，保留两 child 顺序与唯一性；负例不能降级成单 child 成功。
- [ ] 2.2 复用 `A` 的声明型证书，为接口核 flags、无构造/字段/Signature、完整无 Code 的两个抽象方法和无根使用；证明两份物理报告均完整，预算/取消原子停止。

## 3. 根源码一次性投影

- [ ] 3.1 以同一家族状态承载两份已证 relation/child/声明，而不另建脱离 family 的源码捷径；在一个 class-source checkpoint 中发布两个嵌套声明及各自派生来源，失败整组拒绝。
- [ ] 3.2 `I` 的方法保持合法接口分号声明，`A` 保持原抽象类构造与抽象方法；保留物理 child 可查询。核名字不硬编码、顺序、完整来源、无重复 root 使用及输出预算/取消。

## 4. 三方验收与回归

- [ ] 4.1 用 fresh CLI 比较原/JADX/Jarde 的完整 `Shape` Java 8 源码与同一 Runner，`java -Xverify:all` 行为和反射一致；所有有效近邻保守拒绝，不把 `Generic.A` 误投影。
- [ ] 4.2 运行现有 `SingleAbstract.A`、构造型 static member、嵌套 enum/annotation、member family 与 class-source 测试，`cargo test -p jarde --test class_source --locked`、`cargo check --workspace --locked`、fmt、OpenSpec strict、diff check；清理本任务专用 Cargo target。
- [ ] 4.3 root 独立审阅联合关系、接口声明、原子来源、三方运行和负例，更新 EM-01 清单并写验收记录；仅验收 `Shape` pair，`Generic.A` 另列。
