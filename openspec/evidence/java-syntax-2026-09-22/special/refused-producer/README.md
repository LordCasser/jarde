# 特殊调用拒绝的独立反例

脚本以本仓库自写 SpecialProbe.class 为输入，只做同宽字节修改，不增加依赖或永久 class。两个 patched class 均已由 `java -Xverify:all` 执行原 fixture 的 SpecialRunner，JVM 接受，不是利用坏 class 在更早阶段退出制造的空断言。

- `make_probe.py`：把 privateHelper 标为 public，把 superWithSideEffect 的 invokespecial 目标改为该同类成员 `(I)I`。原程序实参依然运行一次，返回 3；首版 special 修复正确拒绝非 private 同类目标，却只引用 BCI 7/4，丢掉 BCI 1 的参数调用。结果保存在 before-original.txt、before-jarde.java.txt。最终重建 CLI 后，after-jarde.java.txt 记录 `@bytecode 7 4 1`，所缺来源已保留；永久测试同时断言三处 source map。
- `make_other_receiver_probe.py`：把 callOtherPrivate 的 invokespecial 目标改为父类 valueWith `(I)I`，实际 receiver 仍是 arg1。JVM 执行 other=12；该形状不能写成以当前对象为接收者的 `super.valueWith`，新恢复规则保留引用并报告非入口 this。结果为 other-original.txt、other-jarde.java.txt。

从仓库根目录运行脚本。执行 driver 时，把 BaseProbe.class、DefaultProbe.class 与 patched SpecialProbe.class 放在脚本指定临时目录，再对 SpecialRunner.java 使用 `javac --release 8 -g:none -cp <dir> -d <dir>`，最后 `java -Xverify:all -cp <dir> SpecialRunner`。恢复只读取该目录中的 SpecialProbe.class，用 single-class/release 8；未启动依赖展开。
