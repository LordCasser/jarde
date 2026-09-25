## 1. 固定错误行为

- [x] 1.1 将自写 SpecialProbe、父类、直接接口和执行 driver 收成独立 fixture，先验证现有输出造成 StackOverflowError、jadx 的接口调用返回 7 而原 class 返回 11；在 verification.md 保存源码/字节码/两个反编译器摘录及重放命令。

## 2. 保留目标选择证据

- [x] 2.1 借用同一 MethodIr 已有类头中的直接父类与接口，保留 CallTarget 的池项类型，验证单方法/类源码以及 class/jar 入口均使用同源事实，不增加类头/方法体读取。
- [x] 2.2 按 design 分别构造 class-super、interface-super、本类 private 的接收者；保持构造器及普通调用。验证返回 8/11、this/其它 private 接收者、入口 this 证明、来源 BCI、缺失目标/接收者证据时 fallback 的正反例。

## 3. 执行与独立验收

- [x] 3.1 对完整恢复的文本重新 javac，执行覆写、接口默认、私有调用、null 接收者、参数计数与参数异常对照；输出与原 class 一致。不得把 jadx 的错值作为期待。
- [ ] 3.2 主代理审查、复跑调用/构造/类型/预算邻近回归，集中更新 fixture census/fingerprint，执行 fmt、受影响包 clippy 和 OpenSpec strict validation，更新原大 change 的 4.3 指向本项；没有实测通过的项目不得勾选。
