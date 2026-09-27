## 1. 标量 String 证书

- [x] 1.1 核对固定 `StringTernaryInit` 的三个 Code diamond、构造器 Signature/字段/BCI 与现有 int、varargs 证明入口；以独立标量字面量正例和错 descriptor/字段/构造器负例验证准入只认准确 `(String,int,String)`。
- [x] 1.2 为同类 `()Z` 条件、`ifeq/ifne`、双 ASCII String literal arm、唯一 `goto`/构造器汇合建立有界值证明；用正反极性及额外 arm 效果、嵌套 branch、错目标/消费者、handler、非 ASCII 负例验证值类型与执行顺序。
- [x] 1.3 将标量 String 参数与构造器中唯一同 owner String 字段存储、完整常量组和物理来源闭合；用错 store/额外效果/第二常量失败、预算和取消测试验证不发布部分 enum 投影。

## 2. 源码投影与三方验收

- [x] 2.1 在普通 enum 源码装配中消费已证 `SingleString` 标量值并输出准确构造器声明/常量参数；编译固定 `StringTernaryInit` 完整 Jarde 类源码与 runner，以 `java -Xverify:all` 核对 `string-ternary=A:B:2`。
- [x] 2.2 增加能交替选择 true/false String arm、计数每次 `()Z` 调用的 Java 8 fixture；原 class、固定 JADX、Jarde 完整源码分别重编并以 `java -Xverify:all` 比较字段值、调用次数和顺序。
- [x] 2.3 重放普通/literal/int-ternary/`String...` varargs 控制及 source map/物理成员查询；运行相关 Rust 定向测试、`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check` 与 `openspec validate recover-string-ternary-enum-arguments --strict`，记录结果并清理专用 Cargo target 供 root 独立验收。
