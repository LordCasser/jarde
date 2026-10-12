# 合并控制的无调试信息观察

root 用同一已冻结原源码、旧产品 CLI 和工具链实际运行 `-g:none` 对照，25 条命令，Java 8/23 原类编译运行 exit 0。原始 javap 没有 LineNumberTable、LocalVariableTable 或 LocalVariableTypeTable，方法 Signature 仍保留。17 条 oracle 与 debug 原类一致。JADX 的 raw-alias 产物和旧 Jarde 的局部类型/handler 产物仍整类编译失败；失败不运行、不算通过。完整原始记录见 `capture/`，成功独立观察见 `observation.json`。

collector 只新增明确的 debug/none 选项，默认仍是 `-g`；所有实际编译命令按选项使用 `-g:none`。root 逐行审查 Luna 私有改动，复制同字节工具后运行，输入/工具 before/after pins 一致。首次 checker 实际 exit 1：无调试信息时异常表后紧接 `StackMapTable: number_of_entries = 4`，原解析器只认没有后缀的属性名。失败保留在 `failed-v1-checker-invocation/`，没有重跑 collector 或修改 capture。

fresh v2 checker 仅增加该有限属性头的精确数字匹配。仍逐方法比较物理 Code 全部异常表、javap 表和 parser count，并保留确切 source/owner/method 的 handler 准入。root 再次独立复制、实际运行 exit 0；状态 `verified_observation_capture_with_quality_gaps`，四个 handler method map 缺口仍如实报告，`behavioral_acceptance=not_claimed`。这里是旧产品观察，不是本次泛型实现通过。
