## Why

DT26 现有 P02_multianewarray 两条冻结输入的数组捕获已恢复，但 lambda helper 中 `t[0][0] += i` 仍引用 bytecode，生成的完整类无法编译；原程序两腿均输出 `6\n`。现有 compound 证明在 dup2 复制值上提前要求类型，复制值不保留 aaload 的数组来源，而后面的原始输入类型与准确复制身份证明已能闭合，宜修正证明位置而非新增恢复机制。

## What Changes

- 对现有 int 元素加法 compound 路径，依据 dup2 原始行数组及四个准确复制 ValueId 证明二维/三维数组元素更新。
- 保持 iastore/iaload/iadd、唯一消费者、区间、求值顺序、预算/停止及原始来源要求；使用现有 IndexAssign 与数组来源追溯，不扩大操作符或 primitive 家族。
- 精确升级旧 P02 helper 断言，保留历史 baseline；新增完整类求值次数、异常次序与 RHS 改行控制，在真实 JDK8/23 对照源码、jadx、jarde。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已有 int 数组 compound 更新可在原始行数组来源与 dup2 复制身份闭合时恢复嵌套元素。

## Impact

限定 `crates/jarde-java/src/build.rs` 既有证明、生产测试、完整 fixture 和验收证据；不新增 pass/AST/类型服务/依赖，不修改 lambda/SAM/capture、concat 或其它 primitive 更新。BigDecimal 产品确切 CI 待验收期间保持旧产品源码身份；本片准备与新产品验收分开。71 单元分母和 DT26 整单元分类保持，不以窄片代替全单元验收。
