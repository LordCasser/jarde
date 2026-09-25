## 1. 固定字段与命名边界

- [x] 1.1 从 expanded 审计收成最小 Java8 fixture 和 source-only helper/runner，覆盖顺序、分支、局部与后缀重名、先赋后读；保留 ConstantValue、实例 final 和接口的独立对照。记录 hash/Code 数及修前红测试，永久语料统计由 root 统一更新。
- [x] 1.2 固定缺失/不匹配字段声明、非 final、其它 owner、同名歧义的负面测试，确认新判断不从 CP 名称或类名猜测；代码审查确认字段头直接借用同一 MethodIr.facts 中的 fields slice，没有复制或重新读取。

## 2. 闭合简单字段赋值

- [x] 2.1 在既有声明/字段路径中判断本项受限写入，将相同前提决定的名称约束交给既有 NameTable，覆盖 local0、local0_2、debug 范围名与 free_name，确认普通方法命名回归不变。
- [x] 2.2 让现有 FieldAssign 以可选 receiver 表达简单字段名，适配既有遍历/发射，并只在已证明的原 putstatic 处生成；字段、accessor、构造器与类型转换相邻回归通过，不增加 pass 或上提表达式。
- [x] 2.3 验证真实赋值/生产者 BCI 与成员来源、默认无来源、正文不足及来源不足，commit/replay 一致且预算停止保持既有产物契约。

## 3. 实际执行与独立验收

- [x] 3.1 原样重编译实际恢复完整类，以独立 JVM 对照原 class 的两分支、字段值、顺序、计数和异常；记录 JADX 完整输出对照，不修改生成赋值或删方法后报通过。
- [x] 3.2 root 审查实现、执行 expanded 独立输入并复跑字段/命名/初始化/来源预算回归，集中检查 census/fingerprint、fmt、clippy 和 OpenSpec strict；将接口等范围外债务与本项失败分别记录。
