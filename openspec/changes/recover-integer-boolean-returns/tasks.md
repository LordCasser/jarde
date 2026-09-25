## 1. 冻结真实边界与三方基线

- [x] 1.1 记录现有 `(I)Z` `iload; ireturn` 补丁类的源码、descriptor 改动、Code/hash、原 JVM 奇偶结果和 JADX/Jarde 原样完整输出阶段；复现命令与冻结文件 hash 写入本 change 的[分析记录](analysis.md)，确认 `(Z)B/C/S` raw 2 是独立负例。
- [x] 1.2 固定只含本项可恢复方法的最小 Java 8 source-only 类、精确 descriptor 补丁和 runner，保存原 class/JADX 原样阶段并说明并行生产改动下 Jarde 修前阶段的可得性；以 `javac --release 8`、`java -Xverify:all`、补丁 Code/hash 与直接/调用/字段前后自增/同步的值、次数、异常验证。真实共享 switch `ireturn Z` 另设一份受控边界样本，不把条件 stack phi 算作正例。

## 2. 在已有返回消费处表达最低位

- [x] 2.1 复用或提取 Z 字段写入已有最低位 AST 构造，只在真实 `ireturn`、方法 `Z` 和已呈现 B/C/S/I 同时成立时适配返回；以 1.2 的奇偶、负数、极值与非 `Z` 相邻回归验证，不放宽普通赋值或调用。
- [x] 2.2 普通、同步、switch 下推和已证明字段自增返回共用实际返回位置适配，保留已证 boolean/0/1 快路与值原求值位置；以真实 operand/`ireturn` source map、字段完整更新、一次调用及同步 null 的执行结果验证。
- [x] 2.3 固定未知/不相容值、`(Z)B/C/S` raw 2 及 boolean 参数调用的现有拒绝；以默认/all/replay 相同正文、受限预算/取消无半成品和来源/延期生产者回归验证。

## 3. 三方完整执行与 root 验收

- [x] 3.1 原样生成零引用完整 Jarde 类并用 Java 8 重编、严格验证执行，与永久原 class 逐行比较值、字段状态、次数和异常；保存 JADX 的原样 `javac`/运行阶段，不修补其输出后续测。
- [x] 3.2 root 独立复放 1.2 和共享 switch 样本，审读真实 `ireturn` 准入、最低位 AST、求值顺序及来源，并复跑 boolean field、boolean contexts、窄整数 return、field increment、sync/switch、deferred-order 相邻测试；其它条件 phi/类型推理债务按 roadmap 分案。
- [x] 3.3 root 统一更新 fixture 索引与 corpus fingerprint，运行本案 Cargo/JDK、fmt、适当 Clippy、`git diff --check` 与 `openspec validate recover-integer-boolean-returns --strict`；记录全工作区既存门禁缺口，并在停止构建后清理私有 Cargo target。
