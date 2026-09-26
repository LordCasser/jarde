# 成员局部声明源名验证

## 冻结基线与全类闭环

输入复用 [冻结 OuterSuperEffects 证据](../../evidence/java-syntax-2026-09-27/outer-super-bridge-effects/analysis.md) 的三个 class 文件；SHA-256 为 `EffectsBase.class` `b2ec9055704f161efb71e72972cb0827750257c172d504ec6a311fd2b5fbbf52`、`OuterSuperEffects.class` `7b2dea2282f4909a882ce6495feb8387a6088e89cc59616ca2e4dd0e9d12d3a8`、`OuterSuperEffects$Member.class` `02534b17440840aa452b3a4276f79d40a4e2ead66ea96a4115ef7bed6474c266`。既有 [Outer.super 阶段记录](../project-proved-outer-super-bridges/verification-3-in-progress.md)准确记录了修复前唯一整类编译阻塞：外层 `main` 仍输出 `OuterSuperEffects$Member`，而成员家族声明了嵌套类型。

当前无修改生成闭环使用：

```text
CARGO_TARGET_DIR=/tmp/jarde-member-local-types-target CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo build -p jarde-cli --locked
jarde-cli class-source --input /tmp/jarde-member-local-types-cli/input.jar --class OuterSuperEffects --policy plain-jar --release 8 --format text
javac --release 8 -Xlint:-options -g:none -d <classes> OuterSuperEffects.java EffectsBase.java
java -Xverify:all -cp <classes> OuterSuperEffects
```

原始 Java、冻结 JADX Java 和 Jarde 完整家族文本均以 Java 8 编译并在完整验证下运行；三方输出一致：`12:123`、`fail:1`。Jarde 输出同时包含 `OuterSuperEffects.Member member = outer.new Member();`、`OuterSuperEffects.super.combine(...)`、`tick(1, failFirst)` 与 `tick(2, false)`。CLI 输出未作手工编辑。

## 拒绝与请求边界

`local_declaration_source_name_requires_one_complete_nongeneric_member_path` 覆盖：有完整且唯一的非泛型成员路径时映射为 `pkg.Outer.Member`；合法顶级 `$` 名、缺失路径、同名物理定义歧义、泛型成员路径和冲突源段均返回无别名。该单测构造既有成员事实，不伪装成完整 class-file 集成反例。

`stopped_class_source_does_not_publish_partial_member_spelling` 用 1-byte 输出预算与预取消分别验证停止路径；输出停止的报告不得声称完整且不含被投影的成员声明，预取消结果为 `Incomplete`。`standalone_method_recovery_does_not_inherit_family_source_aliases` 从类报告取 `main` 的物理方法身份后发出独立方法请求，确认身份未变且该方法结果不借用家族局部类型别名。默认与 `all` evidence 的 class-source 文本相等，由同一集成测试验证。

## 定向验证

以下命令均使用隔离 target `/tmp/jarde-member-local-types-target`，并通过：

- `cargo test -p jarde-java a_delimited_or_already_grouped_position_gains_no_parentheses -- --nocapture`
- `cargo test -p jarde-java local_declaration_source_name_requires_one_complete_nongeneric_member_path -- --nocapture`
- `cargo test -p jarde --test member_local_type_source_names -- --nocapture`（3 项）

发射器测试覆盖普通声明与 `for` 声明使用源名，并要求对应 source-map span 含声明文本；断言尊重既有同 BCI span 可能覆盖完整语句的契约，没有收窄发射边界。
