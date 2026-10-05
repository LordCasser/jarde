# java.util.Optional 链巡查（2026-10-05 root）——critical 第 17 锚 + **第 5 诊断族**

## critical 第 17 锚（新族：绑定接收者适配）：`o.ifPresent(sb::append)` 调用整个吞掉

sideEffect（**ifPresent + 方法引用副作用——Optional 最常见用法**）：`o.ifPresent(sb::append)`（绑定接收者=捕获局部 `sb`）——**ifPresent 调用语句被完全吞掉**，幸存 `StringBuilder sb = new SB(); return sb.toString();`——**隔离编译 exit 0、`[]/[]` vs 原 `[S]/[]`**（可编译错，第一不变量违反）。

诊断文本：**"adapting this bound receiver would move its null failure from functional-value creation to invocation"**（BCI 20 8 9）——**四主族（旧值/copy/依赖链/多消费者）之外的新诊断族**（绑定接收者适配/NPE 时机——DT-27 方法引用 NPE 原则域的**逐值引注形**：此前只见整方法拒绝，本锚为**语句吞掉+方法幸存**的 compilable-wrong 形）。伴随 copy 行（BCI 10/11：allocation/copy 无证明）为级联。jadx 解此形（SB 引用直传）。

## 健康面（负结果）

- **name**（ofNullable+map+filter+orElse 全链 + String::trim 方法引用 + lambda）完整恢复——**绑定到参数的方法引用/普通 lambda 恢复**，失败面收窄到**绑定接收者=捕获局部**；
- len（isPresent+get 老 API）、parse（工厂三分支 try/catch）、main（Optional.of/empty 构造+九段拼接 println）全恢复；
- 行为 `hi/none/none/3/0/42/false/S/` 一致（前四方法）。

## 处置

soundness spec 补第 5 族 Requirement 行 + 本锚 scenario；census 记第 6 条（普查后新族）；账本 17 锚/5 族。
