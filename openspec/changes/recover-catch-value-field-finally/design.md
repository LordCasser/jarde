## Context

[固定 Test7 证据](../../evidence/java-syntax-2026-09-28/cf16-test7-audit/README.md)说明 debug/no-debug Java 8 class 的 `test(Object)` 指令及四行异常表相同：`[0,6)→19 Exception`、`[0,6)→35 any`、`[19,22)→35 any`、`[35,37)→35 any`。正常清理在 BCI 6–16，具名 catch 在 19 保存 Exception 和 false、清理在 22–32；catch-all 从 35 保存 Throwable、37–47 清理并重抛，50–51 从跨 try/catch 的 local 2 返回布尔值。第四行仅保护 handler 的 `astore`，不保护 `f++`。

现有 `SharedFinally` joined completion 能表达 Test3 的具名 catch 与四行自保护，但其四行实例清理证明限于两指令 `unload()`，join 为 void 返回。Test7 的清理为 `aload_0; dup; getfield f; iconst_1; iadd; putfield f`，join 消费 local 2 的两个到达定义。现有字段单位更新的 Builder 语法可复用，不能仅比较三段文本或字段名称。

## Goals / Non-Goals

**Goals:** 对固定四行布局证明三份同一字段增量、正常/catch 的返回值合流和异常原值重抛；输出一次 `try/catch/finally`、一次 `f++`、一次尾部 return，完整类 Java 8 可重编且所有 BCI 可溯源。

**Non-Goals:** 任意字段表达式、DEX lowering、Test3 的 `<clinit>` 缺口、未验证的异常 handler 变体，或把 Test7 的单条 no-debug 文本断言看成全形态运行证明。

## Decisions

1. **扩展现有四行共享 finally 证书的完成类型。** Guard 沿 Test3 的 `SharedFinally` 四行行序/自保护/边归属预检证明本类，但为合流后的保存值增加明确的 joined-value 完成契约；旧 void joined 和两份返回的证书保持原约束。无需新 pass/Region/AST；若把 local 2 的两个到达值视作一个通用 slot，会漏掉 catch=false 或额外定义，故必须核 SSA 合流和 terminal load/return。对另一种做法——复制整套四行 guard/region/build——会造成异常所有权和来源逻辑分岔。
2. **字段增量副本独立按语义证明。** 三份各需同一个入口 `this` 接收者、同一个已解析实例 int 字段，`getfield` 的值与常数 1 进入一次整数加法，其结果由 `putfield` 写回该接收者；没有额外消费者、调用或中间副作用。副本等价是物理 CFG、SSA 与操作目标的结论，不是 `f++` 名称相同。清理位于三段异常保护范围之外，handler 的自保护边只从 `astore` 回到自身。
3. **Region/Builder 将两个到达值绑定为一个源码局部。** `exc(obj)` 的结果及 catch 的 false 都在清理前写入 local 2，尾部从相同合流值返回；Region 保留 try/catch 的两条完成路径，Builder 在局部声明、所有块/BCI 来源及预算可呈现后提交一次 finally 与尾部 return。任一分支 fallback、源值不明、额外入口或取消时回滚并保留物理拒绝，不留半个 try。参考 JADX 寻找三份清理的遍历顺序，不照抄其按相似指令消隐。
4. **验收区分默认 DX 与固定 Java-input。** 上游默认集成测试走 DX，使用原 `check()` 行为说明正例；本变更对固定 Java 8 classfile（debug/no-debug 相同 test 字节码）进行 Jarde 证明。原 class、转写源、JADX Java-input 和 Jarde full class 用 `javac --release 8` 与 `java -Xverify:all` 对照；为具名 catch 路径可只改 `exc` 的 probe 体，保留被验收的 `test` 方法字节码与四行表不变。

## Risks / Trade-offs

- **`f++` 读写链被误判为同字段副本** → 同时核 resolved 字段、入口接收者、SSA 生产消费、唯一副作用及 verifier 有效近邻。
- **local 2 两个定义在 finally 后合流** → 显式校验 phi/局部生命周期与尾部消费；任一 branch 不能呈现时整体回退。
- **handler 自保护范围误含字段写回** → 固定 `[35,37)` 并核每个 throwable site/异常边，扩围近邻拒绝。
- **profile 与行为证据被混用** → 默认 DX 测试与独立 Java-input classfile 的证据等级分列；只将固定 Test7 classfile 切片计为恢复。
