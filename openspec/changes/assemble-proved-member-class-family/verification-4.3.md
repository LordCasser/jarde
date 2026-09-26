# 4.3 外部消费者与拒绝边界

家族源码隐藏捕获字段和构造器隐式首参前，现在使用现有的 `declaration_references` 对两个精确 `ResolvedMemberRef` 作全量、同预算的物理引用扫描。扫描包含字段/调用指令、常量和 bootstrap 消费者；只接受捕获证书中的构造写入及 child 读取 BCI，和构造调用证书中的精确 caller/BCI。任何其它物理使用、未决候选、覆盖不足、分页、停止或环境不完整都拒绝家族投影，原因保留物理使用者位置；根与 child 已恢复的物理文本、方法及来源不变。

[`member_family_external_use.rs`](../../../tests/member_family_external_use.rs)在 Java 8 下先编译同二进制名的静态成员 stub 和 `ExternalUse`，分别生成对 `Member.this$0:LNamedMemberFamilyStage1;` 的 Fieldref、对 `Member.<init>(LNamedMemberFamilyStage1;)V` 的 Methodref；`javap` 检查常量池后，将 stub 换成冻结的根/成员 class。两种组合均用 `java -Xverify:all` 加载 `ExternalUse` 成功，是有效的外部使用负例。Root 用独立 Cargo target 复跑四项测试：两种外部引用均 `Refused` 且原因包含 `ExternalUse`，无外部者仍 `Projected`，`archive_entries=150` 令引用扫描停止时保留两份物理报告并拒绝投影，4/4 通过。

此前[4.1 验收](verification-4.1.md)和本轮 14/14 家族回归覆盖冻结 `20:10:1:3` 的 `Outer.super` 桥、关系错配、未恢复正文及预算/取消；它们仍保持拒绝，没有将普通调用冒充 `Outer.super.value()`。Root 对本轮生产代码复跑 `cargo test --locked -p jarde --test class_source --test member_family_identity --test member_family_external_use`（47+14+4）和 `cargo test --locked -p jarde-cli --test class_source_cli`（17），全部通过。

闭包证明仅限请求选定的 `PhysicalScope`。本阶段只放行单一普通 JAR、一个加载根和关闭 multi-release 的环境；多 snapshot、其它加载根或多版本视图保守拒绝。它不声称证明反射、未提供的外部客户端或运行期改写不存在。更宽的可见范围需要另案证明，不能把本范围的完整扫描解释成开放世界安全。
