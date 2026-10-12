# wrap Load 回退修复与最终局部验收

workspace-v6 的真实 C2W.main 回退已单独保留。根因是 catch 参数来源证明新增的全局 Caught 拒绝，误把另一 handler 的独立绑定当作当前参数逃逸。修复仅撤回该分支，按现有 handler 入口 store ValueId 认证；新 Load 的逐 handler Store/Load/Phi 别名证明、精确声明类型门、空/缺失 Phi 与 Itself 拒绝保持。SSA Caught 来自 throw site 的 Stack(0)，进入 catch local 须经该 handler astore。Luna 私有 patch/可达性审计及 root 应用后的源码快照全部保留。

focused-root-v10 实际 8 命令全部 exit0，518 passed / 0 failed / 4 ignored，4854 来源前后不变，当前 CI Clippy、fmt、global strict、CLI 构建及 finally exact clean 均成功，target absent。冻结 CLI SHA c8c18f26a256027262b514455c33f24d0ae1cbcec658e3d6ab76b2c3a9c2ccfd，receipt SHA722c0c738a85fc03cddb2249049f4fd5087002abe0cdd4e35c4df5173acbbf74。原 scope SHARED/ESCAPED 和 wrap-family 断言不变；root独立验收器接受所有来源与具名门。

新父项对照实际40命令/16 renders/12未改写整类运行同原；字段对照24命令/4 Jarde+4 JADX整类同原；TWR23命令，原/JADX/父项/当前双JDK同原。九步编排实际全部 exit0。匿名对照29命令，已有构造器债务仍整组compile-failed/no-runtime，三份生成源码与上一候选逐字相同；不声称整个物理类通过。

新增 C2W 完整类对照实际27命令：原 frozen class、fresh JADX default/none、Jarde共8份双JDK编译成功，随后16次-Xverify:all运行。C2WRunner全部等于永久Rust oracle；实际C2W.main全部逐字等于同JDK原class，覆盖连续catch复用slot1。完整生成源码未修补，全部static方法及两个pick overload保留，Jarde源码与CLI raw响应text逐字相同。receipt SHA f81e64899de9dce803f8f34c6efd58ba9cc161c6fe55ded0eab1e10204a12df4。

归档包含完整 raw/source/compiled class/collector/checker/pins；不收录62MiB CLI二进制，保留其path/size/SHA冻结身份。root已逐成员复开核验。这里只接受局部与完整类对照，fresh workspace双seed、精确产品自身CI、提交推送和干净main仍待完成，不能拼接失败批次或增加71/612完成数。
