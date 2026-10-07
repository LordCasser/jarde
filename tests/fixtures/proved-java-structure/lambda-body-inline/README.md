# Lambda 多语句体内联正例

`LambdaBodyInline.java` 捕获 `build` 的稳定局部参数，并在 lambda 箭头块里依次记录开始、增加计数、记录结束，最后返回计算值。`build(capture())` 在创建期间只执行一次捕获表达式；断言确保这时 lambda 正文尚未执行。随后两次 SAM 调用各执行完整正文一次，输出包含精确事件顺序、计数与返回值。

`IntAction.java` 是独立顶层 SAM 接口。源码不依赖嵌套类型，也不访问其它类的私有成员。目录中的 `.class` 是 `javac --release 8 -g` 从这两份源码生成并冻结的唯一副本。

`run.sh` 会在临时目录重编译、以 `-Xverify:all` 运行，并在退出时清理临时产物。

## 行为基线（CI 守卫实测，2026-10-07）

`java -Xverify:all` 运行冻结 class（JDK 23.0.1）的输出：

```text
created captureCalls=1 bodyCalls=0 events=[capture]
first result=12 bodyCalls=1 events=[capture, start:10:2, end:10:2]
second result=13 bodyCalls=2 events=[capture, start:10:2, end:10:2, start:10:3, end:10:3]
```

CI 守卫见 [`tests/fixture_behavior_guards.rs`](../../../fixture_behavior_guards.rs)：呈现腿在默认套件，
行为腿标 `#[ignore]`（`cargo test --test fixture_behavior_guards --locked -- --ignored`）。本 fixture 的
渲染文本当前**不可编译**（`main` 未恢复，留 `jarde_refused_body();`；箭头转发到改名 helper
`lambda$build$0$jarde`），行为腿钉的是"javac 非 0 + 记录诊断"这一事实，不当作行为已验证；5.2 落地把
多语句体放进箭头时，须在同一次提交更新该锚。取证与分类见
[`openspec/changes/recover-fixture-behavior-guard-coverage/results/`](../../../../openspec/changes/recover-fixture-behavior-guard-coverage/results/README.md)。
