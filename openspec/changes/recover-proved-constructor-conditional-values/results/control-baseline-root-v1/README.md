# 构造条件值旧 CLI 控制基线（实际观察）

Root 接受 39 命令/78 raw，双 JDK 原类及 JADX default/none 四组完整类共六 runtime 腿的 19 行输出完全相同，含对象身份、调用顺序、null 构造失败、字段赋值失败与 catch。Jarde default/all 共 12 次 combined/isolated 编译失败，没有 Jarde runtime；隔离仍使用相同生成源码，不算独立输入。不编辑生成 Java。

旧 CLI 为上一已交付产品的 final-cli-root-v3，绑定 final-lint-build-root-v3 的 4718 完整输入闭包。本次不是新实现验收。Collector 成功，v5 checker 因无 handler 假设实际失败，保留原 launcher failure/原 raw；v6 只补物理 Code 异常表与 javap 有序完整表、BCI 边界核验，独立执行 exit0。v6 仍明确 quality gaps/behavioral acceptance not claimed，root 仅接受旧观察及原/JADX oracle。

所有文件见 root-acceptance.json 的逐项归档 hash。无 Rust 构建，target 一直不存在。
