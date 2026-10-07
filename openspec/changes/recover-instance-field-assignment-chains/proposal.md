# 实例字段赋值链（recover-instance-field-assignment-chains）

## Why

[chained-field-assignment 探针边界](../recover-chained-field-assignment/results/02-probe-instance-chain.txt)（已记录未实现）：`this.a = this.b = this.c = 5` 整方法拒——javac 发射 `aload_0 ×3; iconst_5; dup_x1; putfield c; dup_x1; putfield b; putfield a`，**`dup_x1` 的拷贝既是 store 的 value 又穿插于栈上 receiver 之间**（静态链的 `dup` 无 receiver 在栈）。已落的 `FieldCopies::Chain` 只读 `dup` 形（value 跨静态 putfield 存活）；实例形的 receiver 交织使同一 proof 读不到该舞蹈。jadx 有解；真实代码中 setter 链/默认值批量赋值常见。

## What Changes

- `FieldCopies::Chain` 增**实例形**：`dup_x1`（value 插入 receiver 之下）跨 n 个实例 putfield 存活——判据同静态链（单源已求值、全部消费是 putfield、求值一次）+ 每一 putfield 的 receiver 是**本 `this` 的同一 SSA 值**（栈上三个 aload_0 同一性）；
- 呈现：按字节码序（源求值序从右到左）n 个独立实例赋值 `this.c = 5; this.b = 5; this.a = 5;`（与静态链同构——jadx 同形）；
- 静态链/`Receiver` 复合/其余 FieldCopies 判据逐字不动。

## 硬不变量

1. 静态链锚（CH）与复合锚（SC/BF）渲染逐字节不变；
2. receiver 非同一 SSA（如 `o1.a = o2.b = 5` 跨对象链）保持拒绝——MVP 只收 `this` 链；
3. 求值一次语义（纯源共享同一已求值表达式文本）。

## 验收

- 探针 CP.inst 恢复（0 该族诊断），整类重编 exit 0、`-Xverify:all` 行为一致；
- 门控实验先行（只开实例形）；全门禁 + oracle ignored 腿 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：实例字段的链式赋值按求值序呈现，方法行为完整。
