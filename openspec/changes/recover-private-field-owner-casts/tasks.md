## 1. 固定单 setter 缺口

- [ ] 1.1 纳入 `dt29-reference-cast-audit/fixtures/private-field/` 与回放生成的三方快照；确认唯一 setter 同时暴露父类直接写入、`B→A` accessor 参数转换及 private helper 正文缺口。

## 2. 恢复已证明的两种字段写入

- [ ] 2.1 对精确 `A` owner 与已证明 `B extends A` 恢复 `visible` 写入；owner/descriptor 错配及缺少继承关系时必须拒绝，并保留 field-instruction 来源。
- [ ] 2.2 只对唯一、无额外效果的 javac private-field setter accessor 恢复 `hidden` 写入、`B→A` 调用实参与结果丢弃；多个目标或其他字段写入必须拒绝。

## 3. 三方闭环与组合边界

- [ ] 3.1 原/JADX/Jarde 的 private-field 完整 class-source 使用同一反射 Runner 经 `javac --release 8` 和 `java -Xverify:all`，输出必须是 `true:false`；接口闭环必须继续输出 `runnable:ClassCastException`。
- [ ] 3.2 重放 owner 错配、accessor 歧义/附加效果负例；更新三方源码与哈希。fixed `TestFieldCast` 复杂组合另作集成待验，不把其余缺口并入本 change。
