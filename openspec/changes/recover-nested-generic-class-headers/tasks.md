## 1. 取证与基线

- [x] 1.1 重放固定 Z1（SHA 核对）：读 `class_generic_source_unproved` 产生点（未证位分解）与 scope 注入路径；记录 Box 三重阻断链基线。（产生点在消费层 `src/class_source.rs::project_generic_signature`——`$` nesting 位合并拒绝；顶层已证 name/kind/父身份/擦除/type-use 位；嵌套位数据源=类自身 InnerClasses 自表行（与折叠片子侧判据同源）；scope 注入=既有 `generic_scope`→成员投影输入。SHA 复核与基线输出见 `ngh/results/before/`，未证位分解见 `ngh/README.md` §1）
- [x] 1.2 构造并冻结至少四个 verifier 有效变体/负例：非静态嵌套泛型、双参 `<U,V>`、bound 形、行不可拼负例（保持阻断链）；各自 `java -Xverify:all` 通过并记录实现前后行为。（NG1 非静态/NG2 双参/NG3 bound/NG4 Z1 同形/NG5 折叠家族/NG4neg 行不可拼负例，全部 `--release 8 -g:none` 且原 class `-Xverify:all` 通过；前后输出与 SHA 见 `ngh/results/{before,after}`、`ngh/results/replay-sha256.txt`）

## 2. 头投影与域链

- [x] 2.1 嵌套位证明（design 决策 1）+ scope 注入（决策 2）；Z1 两口径呈现 `Box<U>`/`U value`；折叠防线评估如实记录。（`member_inner.rs` 提取共享判据 `source_spellable_member_row` + `prove_nested_member_position`（自表行：无 EnclosingMethod、行唯一、`[outer,$,name]==this_class`、行 kind/flags 可拼、行/头 access_flags 对应）；分离口径 `class Z1$Box<U>` + `U value`；字段层 `$` 位以"类头已投影"为证明输入；防线精确化后头投影成功的子类折叠携带 `<U>`——NG5 `static class Box<U>` 折叠成功，Z1/NG1-4 折叠拒于折叠片既有 token 锚定层（外围方法中折叠成员类型的局部变量声明，非泛型 `Solo` 局部同样拒绝，实证登记），见 `ngh/README.md` §3）
- [x] 2.2 负例阻断链原样；既有泛型切片与折叠测试零回退；预算/取消不变。（NG4neg 分离输出与基线逐字一致；`generic_static_member_family` 一处"泛型子类仍拒折叠"预期随本片边界前移更新为折叠并携带 `<T>`（有意变更）；全仓 290/291 套件绿 + 1 已知 flake 复跑绿；预算停止保物理头有专测）

## 3. 回归与验收

- [x] 3.1 全仓测试全绿、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。（2880 通过 + 1 flake 复跑绿；fmt/clippy(29 项 -A) 绿；openspec 256/0；真实命令与结果见 `ngh/results/gates.txt`）
- [x] 3.2 Z1 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。（Z1 子单元替换运行 `b`/`[1, 1]`/`x` 与原 class 逐字一致含 lambda；NG2/NG3/NG4 子单元替换、NG5 折叠根整编全部一致；固定 JADX(dev) 六家族 `--release 8` 重编均败于其泛型 lambda 投影缺口（lambda 片 Y1 同族）如实归档；NG1 非静态子单元替换腿因 javac 内部类仿真不适用，行为锚=原 class + 回归测试反射核对；SHA 见 `ngh/results/three-way/`）
- [x] 3.3 root 独立复核位证明、scope 链与三方行为，更新 DT/EM 账本与巡查记录。（root 于合并主线 c93ced53 复核：分离口径 `class Z1$Box<U>` + `U value;` 呈现、替换重编 `-Xverify:all` 运行 `b`/`[1, 1]`/`x` 与原 class 逐字一致；全仓 2887/0〔`p3_two_exit_return` 目录碰撞 flake 单测绿〕、fmt/openspec 257/257〔一次瞬时竞态重跑排除〕；corpus 2135/2138 逐字同、3 差异全归类。`prove_nested_member_position`（自表行+行/头 flags 对应）与共享谓词 `source_spellable_member_row` 复核认可；Signature 防线精确化（不携带未投影 Signature）接受。**两项裁决**：(a) type-use 注解头位保持整头拒绝——无正例需求不建第二套，登记债务待真实案例；(b) 折叠路径的外围局部变量声明 token 锚定层为独立既有缺口（非泛型，`Solo` 同拒实证）——另片登记。同类绑定链归用户的 same-class-generic-bindings。）
