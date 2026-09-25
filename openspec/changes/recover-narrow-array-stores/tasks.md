## 1. 固定数组写入的实际转换

- [x] 1.1 从core-bcs建立最小永久Java8主class，helper/runner source-only；记录源码、6个descriptor/store opcode补丁、hash/Code及147项原class验证执行、JADX和jarde完整阶段结果，root统一冻结census/fingerprint。
- [x] 1.2 补数组表达式、下标和值生产者的次数/异常顺序对照及普通窄局部/常量写入；B/Z未知元素、boolean操作数和一般Z最低位保留合法JVM拒绝变体，必须验证输入合法且拒绝来源完整。

## 2. 在写入位置表达已有窄化

- [x] 2.1 依据真实store opcode、数组元素和presented整数类型，在array_write用现有Cast补必要窄化；测试B/C/S极值、负值、局部回读、同型/widening/常量及Boolean拒绝，不放宽普通meeting_position。
- [x] 2.2 保持三个操作数的最终求值上下文和共享deferred binding；实测null/越界与生产者抛错优先级、调用次数、原数组值及独立语句前后的顺序，所有BCS正例须真实恢复。
- [x] 2.3 新Cast/来源沿用预算、深度和取消规则；数字不相容的拒绝分支复用现有生产者追溯，修复call@4/store@7只剩7/8来源的已实证缺口。验证默认/all正文相同、store与各操作数真实BCI、合法拒绝边界、受限预算停止及replay稳定，不能用新转换类型标签隐藏未知值。

## 3. 完整类与 root 验收

- [x] 3.1 原样重编译执行永久fixture与core-bcs的147项，要求零引用、返回/数组值/异常/次数与原class一致；196项含Z的原始输入保留边界，JADX若javac失败不得称运行通过。
- [x] 3.2 root独立审读准入和来源、重放真实完整类，复跑array/required-conversions/boolean-contexts/field/return/deferred-order相邻回归；字段、Z和局部类型推理债务继续独立处理。
- [x] 3.3 root统一Java包、census/fingerprint、fmt和OpenSpec strict，记录严格clippy既存region债务；验证完成后才勾选实现任务，不将语料可解析等同于恢复通过。
