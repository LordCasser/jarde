# 匿名根方法带参形 —— 同形编译于无调试信息（`-g:none`）

`recover-anonymous-parameterized-root` tasks 1.2 的双腿对照负腿（design 决策 2）。源文件与
`../anonymous-super-dispatch/` **逐字节相同**，只改编译旗标，故本腿的三个 class 内**没有**
`LocalVariableTable`（`javap -p -l` 计数：root 0、`$1` 0、`Base` 0；`-g` 锚腿相应为 2/2/1）。

冻结的 `.class` 由以下命令产生（在未覆盖既有冻结 fixture 的独立目录内）：

```sh
javac --release 8 -g:none -d . AnonymousSuperDispatch.java
```

本腿钉死的判据：投影的**参数名来自同轮 AST 的参数表**（`class_source_single_parameter_name`
通道），而不是调试信息——`-g:none` 下命名通道自行发明 `arg0`，呈现为
`private static Base create(java.lang.String arg0)` 且捕获读取重拼为 `arg0`；`-g` 锚腿呈现
`captured`（既有命名通道的 LVT 来源，上一环已登记的既有行为）。两腿的捕获读取重拼**机制**
一致（都是"child 的 val$ 读取 → 根方法参数名"），均投影成功、源集 `javac --release 8` exit 0、
`java -Xverify:all` 事件日志与原 class 逐行一致
（`observed=captured-value` / `visibleDuringSuper=true`）。
实现后实测见
`openspec/evidence/java-syntax-2026-10-04/recover-anonymous-parameterized-root/fixed/`。
