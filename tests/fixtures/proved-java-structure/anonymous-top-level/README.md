# 顶层类型匿名捕获正例

`AnonymousTopLevel.java` 自写声明了顶层接口 `Renderer`、顶层抽象基类 `Base` 与公开入口类。目录里的四个 `.class` 是该源码经 `javac --release 8 -g` 编译后冻结的唯一副本。

`run.sh` 在临时目录重编译源码，以 `-Xverify:all` 运行，并在退出时清理临时目录。事件和计数输出验证局部捕获、`choose()`、`Base(long)` 与匿名体调用各执行一次且顺序明确。

## 行为基线（CI 守卫实测，2026-10-07）

`java -Xverify:all` 运行冻结 class（JDK 23.0.1）的输出：

```text
value=23:captured
events=capture,choose,base(23),render
counts=1,1,1,1
```

CI 守卫见 [`tests/fixture_behavior_guards.rs`](../../../fixture_behavior_guards.rs)（行为腿标 `#[ignore]`：
`cargo test --test fixture_behavior_guards --locked -- --ignored`），它另钉子类 `AnonymousTopLevel$1`
的捕获写在 `super(seed)` 之前（字节序；`Base` 的构造器会跑用户代码，越序门不允许越过它）。根方法
`create` 的投影形呈现由环 2 的 [`tests/anonymous_supertype_return.rs`](../../../anonymous_supertype_return.rs)
钉住——本 fixture 的呈现**已由 `recover-anonymous-supertype-return`（合并 `23b69bcf`）从物理文本翻转为投影**。
取证与分类见 [`openspec/changes/recover-fixture-behavior-guard-coverage/results/`](../../../../openspec/changes/recover-fixture-behavior-guard-coverage/results/README.md)。
