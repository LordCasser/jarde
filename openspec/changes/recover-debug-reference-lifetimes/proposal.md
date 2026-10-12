## Why

固定 CF11 `UpstreamNestedLoops.rawAliasEscape` 的两段引用生命周期复用 slot 5：启用 LVT 时，同名 `value` 被合并成一个 Integer，第二个循环的 String 赋值导致完整类编译失败；`-g:none` 已由现有 SSA 证明拆成两个身份。问题在调试记录与既有身份证明的连接，不需要另建类型系统。

## What Changes

- 在现有引用生命周期拆分证明成功后，把非空、互不重叠的 LVT 读取范围和名称一一匹配到已有 SSA 身份；初始化 store 的身份继续由 SSA 决定。
- 保留无 LVT 路径、参数/resource/CFG/alias/预算拒绝条件及现有全局名称冲突后缀。
- 固定双 JDK、真实 `-g`/`-g:none` 的完整类对照；保留 raw List 别名、检查转换、ClassCastException、集合状态及副作用。
- 外层 try/foreach 的 `handled()` 区域缺口另行处理。本项局部改善不能关闭完整类运行门或增加整 CF11/71/612 完成数。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对已有 SSA 引用生命周期拆分附加已核准的调试名称，避免同名 LVT 把不同类型身份合并。

## Impact

主要落点为 `crates/jarde-java/src/reuse.rs` 的私有 helper 与现有 `SlotEvidence::Split`，沿用 declaration placement、NameTable、预算和物理来源。新增永久正反例和完整类证据，不新增依赖、公共 carrier、pass 或 solver。

前置是已证明的 exact reference type、定义/读取所有权和 canonical normal CFG。当前产品基线为 `7e93eb30647338dd8a6ddfc9994392f23457aa3e`，冻结 CLI SHA `c8c18f26a256027262b514455c33f24d0ae1cbcec658e3d6ab76b2c3a9c2ccfd`。两模式各25条真实命令，原类各17场景同原 oracle；JADX也丢失 raw 别名而编译失败。本项不修改泛型投影、不弱化完整结构证明、不承诺两个身份使用完全相同的源码拼写，也不修补生成 Java。
