## Why

CF11 的合法 Object 局部先写 String 字面量、再写 Iterator.next() 的 Object，现有声明决策仍选首写 String，导致完整生成类编译失败。root 已在同次真实 IR 中核准同一 LocalVariable 的两写和 Phi→Load→Object Return 完整消费链；当前 foreach 产品自身 CI 与最终干净 main 已交付，可以处理这个独立类型缺口。

## What Changes

- 在既有声明类型决策中增加一个闭合候选：普通首写定型的非参数局部有准确 Object 写入、全部写均可赋给 Object，且所有实际读取只直接进入 Object 返回位置时，统一声明为 Object；无法核准读取时按既有冲突契约拒绝这个窄候选，不能继续发布已证矛盾的窄声明。
- 复用当前 LocalVariable/uses/SSA/operations、声明与赋值类型结果；核准完整读写和来源，不按名字、slot 或 debug descriptor 推定，不插入 cast，不扩类型层级。
- 用 debug/no-debug 真实 Java 8 class、合法接口与数组近邻控制、参数/slot reuse/handler/Stop 边界验证；新完整源码与原/JADX/Jarde 双 JDK 对照，分别记录编译、运行、来源与拒绝。
- root 先冻结架构与 spec，派 Luna 私有实现，再串行实际构建和对抗验收。只处理该类型窄片，不改 foreach 或 void 循环泛型 Signature 证明，不关闭整个 CF11。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 在既有统一局部类型决策要求中明确准确 Object 写与完整 Object 返回读取的有限兼容候选，防止首个窄引用类型造成矛盾赋值。

## Impact

主要限 `crates/jarde-java/src/build.rs` 既有局部类型决策及必要真实回归；无新 IR、Frame、pass、identity/LUB 服务、依赖或 API。原始普通 LVT descriptor 映射缺口独立记债务。固定71单元/612文件不变；5GiB机器余量/target1GiB、一秒进程组守卫、默认Rust栈、完成clean保持。
