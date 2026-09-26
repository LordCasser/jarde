# 2.1–2.2 身份与同请求准备验证

本阶段只建立物理家族身份和顺序准备。`member_family = prepared` 表示根与 child 的 typed `InnerClasses` 关系、选定定义和两份物理恢复均完整；它不授权隐藏捕获字段、改写接收者或发布嵌套源码。源码文本仍分别采用物理类声明。

- 冻结 `fixture.jar` 的根 `NamedMemberFamilyStage1` 和 package-private `Member`：根 row 与 child 唯一 self row 的 outer/name/flags 一致；成员可见性取 row，child 原始 class flags 不作为 public 门槛。集成测试确认两份不同的 `PhysicalDefinitionId`、各自方法 owner、物理文本和同一预算累计用量。
- proof-unit 变体覆盖根 row 缺失/重复、静态 row、同一 child 的静态冲突、根已有外层 self row、child row 缺失/flags 冲突及 `EnclosingMethod` local 身份。静态 sibling 不应阻断另一个非静态候选；`$` 仅用于核对行所指的字节码名与可重编源码名，不用来发现成员。
- 重打包的 JAR 缺失 child 或包含两份同名物理 child 时，家族状态明确拒绝，根方法和文本仍可检查。child 的 `method_bodies` 预算停止时，状态为 `Refused` 并保留 child 物理报告；根最终 `usage/execution` 包含 child 消耗，child 自身的 `usage` 是同请求较早的累计快照。
- 已取消的 proof-unit 行扫描与零 `analysis_steps` 分别返回真实取消/预算错误，没有把停止伪装成身份反例。

验证命令（隔离 `CARGO_TARGET_DIR=/tmp/jarde-member-family-identity-agent-target`）：

```text
cargo check -q -p jarde
cargo test -q -p jarde --all-features --lib member_inner::tests::family
cargo test -q -p jarde --all-features --test member_family_identity
cargo fmt --all -- --check
git diff --check
```

捕获值流、构造调用、嵌套 writer 和派生源码映射属于后续 3.x/4.x，不能由本阶段的 `Prepared` 状态推出。
