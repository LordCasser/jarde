## 1. 成员属性事实

- [ ] 1.1 在现有 reader 成员属性路径中一次解析 `MethodParameters` 项数、原始名字索引与 flags，并验证完整长度、重复属性、无效索引/flag 的定向 reader 测试通过且预算只扣一次。
- [ ] 1.2 在 class-source 以本成员 descriptor/flags 建立准确参数顺序到局部槽位的证书，验证实例接收者、`long/double` 双槽和计数/空名/冲突拒绝测试；无属性时原有槽位名回归通过。

## 2. 同次命名与声明

- [ ] 2.1 在方法正文恢复前把获证名称交给现有名称表，使声明与所有正文引用来自同一组名字；用无 LVT 的 `named` 全类 Java 8 编译测试确认没有未定义 `arg<slot>`，LVT 控制保持旧路径。
- [ ] 2.2 仅为获证参数位置发射 `final` 并保留 `throws` 等现有声明；用 `-parameters` 重编后的反射测试确认 `paramStr:false:number:true` 及两处 `IOException`，无属性/非法属性不发表半份声明。
- [ ] 2.3 验证属性读取、交接与输出的低预算/取消原子性及物理成员可定位性，运行相关 reader、class-source 和 jarde-java 回归测试。

## 3. 固定三方验收

- [ ] 3.1 用 EM-03 固定 `replay.py` 对原/JADX/Jarde 三份完整 Java 8 源码重编并 `java -Xverify:all`；Jarde 六行输出须与原 class 一致，记录版本、SHA 和负例边界到独立验收文档。
- [ ] 3.2 `cargo fmt --check`、`cargo check --workspace`、相关 `class_source` 测试及 `openspec validate recover-proved-method-parameters --strict` 通过后勾选已完成任务，并清理本次临时 Cargo target。
