## 1. 同轮 LVTT 事实

- [ ] 1.1 理清 reader `MethodCodeFacts::debug`、LVT 名称、Code 嵌套属性和预算边界，在同一读取中保留唯一 LVTT raw Signature 及 slot/范围/名称事实
- [ ] 1.2 对缺失、重复、畸形、与 LVT 不匹配和低预算/取消分别验证，不让 debug 元数据改变真实 Code 成功/停止语义

## 2. 局部声明与菱形证明

- [ ] 2.1 沿既有 facade/`DebugLocal`/reuse/`Declarations` 路径将固定 `Map<String,String>` 与准确 local 身份和擦除配对；决定一次并供所有声明路径使用
- [ ] 2.2 用同轮 Code/SSA 证明唯一无参 `HashMap` 分配、存储、读取及 Java 8 JDK `HashMap<K,V>`→`Map<K,V>` 赋值；对该 AST `New` 写菱形，保留来源与原子停止
- [ ] 2.3 测试第二次写入、slot 重用、其它分配、错签名/范围、无 debug 时不误投影；现有 cast 与调用绑定维持

## 3. 三方验收

- [ ] 3.1 扩展 DT-20 冻结脚本的修后模式：`-g` 保留参数化 `Map<String,String>` 与 `new HashMap<>()`，`-g:none` 保留 raw/cast；原/JADX/Jarde 完整类型 Java 8 重编和 `-Xverify:all` 行为一致
- [ ] 3.2 运行 reader/recovery/类源码定向测试、fmt/check 与 OpenSpec strict，清理临时 Cargo target，并记录本片外的 DT-20 形态
