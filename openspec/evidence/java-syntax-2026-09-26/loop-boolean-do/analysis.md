# 复合 `do-while` 闩锁的两种回边

这是 `present-proved-java-structure` 任务 3.3b 的输入。自写的 [DoLoopBool.java](../../../../tests/fixtures/proved-java-structure/loop-boolean-do/DoLoopBool.java) 有两个体相同的方法，只改变闩锁上的运算符：

```java
do { n++; a--; b--; } while (a > 0 && b > 0);
do { n++; a--; b--; } while (a > 0 || b > 0);
```

用 OpenJDK 的 `javac --release 8 -g:none` 编译；源码 SHA-256 是 `4d2ef462741a2d866cb24e729023730534c19249a7cf447b49d4a86135ff56da`，冻结 class SHA-256 是 `900d52f6329d4ce5240f0cf857a847e791aacf83a4b4d99988f72248b0de9c40`。完整指令见 [javap.txt](javap.txt)。两者每轮都先运行 BCI 2、5、8 的增减，再测试 BCI 12 和 16。`andDo` 的 BCI 12 假边去 BCI 19 出口，真边进入第二测试；只有 BCI 16 的真边回到 BCI 2，所以只有一个物理回边。`orDo` 的 BCI 12、16 真边都回到 BCI 2；前者的假边进入第二测试，后者的假边去 BCI 19，所以有两个物理回边。这是现有 `latch_tested_loop` 只接受单 latch 时不能覆盖 `||` 的直接原因。

## 三方对照

同一冻结 class 的[原源码](../../../../tests/fixtures/proved-java-structure/loop-boolean-do/DoLoopBool.java)、[JADX 1.5.6](jadx.java) 与 [Jarde 当前文本](jarde.java) 已保存。JADX 完整类可用 `javac --release 8 -g:none` 重编；其 `andDo` 仍是 `do-while`，但把第一个失败测试写成体内 `if (i <= 0) break`，只留第二测试在闩锁；`orDo` 则写成 `while(true)` 加 `if (i <= 0 && i2 <= 0) return i3`。两者执行正确，却都没有保留原来的复合闩锁拼写。

Jarde 由 `22bed55e` 构建，以 `recover --policy single-class --evidence source_map` 分别恢复两个方法。`andDo` 报 `jre_region_unmet_precondition` 和 `jre_region_uncovered_blocks`：`loop@1` 是循环规则的版本标识，并非 BCI；该规则在 BCI 2 看到 `iinc`，不能把这条体内效果当成条件值，未覆盖块为 `[15, 19]`。`orDo` 报 `jre_region_loop_shape` 与同一未覆盖块。两者均是 `explanation_only`、整段 `@bytecode 0 2 15 19`，没有可重编的恢复正文。source map 只有一个文本段：primary BCI 0，derived BCI 2、15、19；`andDo` 字节区间为 `[160,330)`，`orDo` 为 `[159,329)`。这至少是可见的拒绝，没有伪称循环已恢复。

以 `java -Xverify:all` 执行原冻结 class，`(3,3)`、`(3,-1)`、`(0,0)` 的两列结果（`andDo,orDo`）分别是：

```text
3,3:3,3
3,-1:1,3
0,0:1,1
```

JADX 重编类的三行相同。运行 [run.sh](../../../../tests/fixtures/proved-java-structure/loop-boolean-do/run.sh) 可重放原类，脚本仅在临时目录生成 class 并在退出时清理。Jarde 在 3.3b 实现前没有可执行正文，不能把当前引用算作执行对照通过。
