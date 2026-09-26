# 带副作用实参的 `Outer.super` 桥

这是 `project-proved-outer-super-bridges` 任务 2.2/3.1 的独立 Java 8 正例。源码分别在 [OuterSuperEffects.java](OuterSuperEffects.java) 与 [EffectsBase.java](EffectsBase.java)。从仓库根目录执行 `javac --release 8 -Xlint:-options -g -d <临时目录> OuterSuperEffects.java EffectsBase.java`（将前两个路径指向本目录）生成并冻结了三份 class；`java -Xverify:all -cp <临时目录> OuterSuperEffects` 输出：

```text
12:123
fail:1
```

第一行证明两个 `tick` 各执行一次且按 1→2 顺序发生，最后才进入 `EffectsBase.combine`（事件码 123），而 Outer 自身覆写若被误调用会留下事件码 9、返回 99。第二行证明第一个 `tick` 抛异常后，第二个 `tick` 和父类方法都未执行。成员方法的 [反汇编](javap.txt)中，BCI 1 从 `this$0` 读捕获 Outer，BCI 6/11 两次调用 `tick`，BCI 14 调 `access$001`；桥 `(LOuterSuperEffects;II)I` 在 BCI 0/1/2 按序加载三个参数、BCI 3 `invokespecial EffectsBase.combine:(II)I`、BCI 6 返回。这不是可用“只接受无副作用实参”覆盖的正例。

冻结 class SHA-256：`EffectsBase.class` 为 `b2ec9055704f161efb71e72972cb0827750257c172d504ec6a311fd2b5fbbf52`，`OuterSuperEffects.class` 为 `7b2dea2282f4909a882ce6495feb8387a6088e89cc59616ca2e4dd0e9d12d3a8`，`OuterSuperEffects$Member.class` 为 `02534b17440840aa452b3a4276f79d40a4e2ead66ea96a4115ef7bed6474c266`。这三份是唯一冻结的 class 副本，后续测试直接引用；临时编译目录不入库。

对三份同样的冻结 class 打包执行 JADX 1.5.6，完整输出保存于 [jadx-generated](jadx-generated/defpackage/)。它在 `Member.run` 写出 `OuterSuperEffects.super.combine(OuterSuperEffects.tick(1, failFirst), OuterSuperEffects.tick(2, false))`，并把 `outer.new Member()` 留在同一外层类 `main`。将两份 JADX Java 用 `javac --release 8 -Xlint:-options -g:none` **完整重编**，再执行 `java -Xverify:all defpackage.OuterSuperEffects`，输出同样是 `12:123` 和 `fail:1`。JADX 的 `defpackage` 包名是其生成策略，不影响此处行为判定。

Jarde 的完整家族投影和三方重编/执行须待任务 3.1 的代码完成后另行验收；当前这份记录只证明原 class 与 JADX 在此形状上的行为基线，不把尚未生成的 Jarde 文本算作通过。
