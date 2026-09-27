## ADDED Requirements

### Requirement: Debug generic locals use same-run type metadata only when identity is proved

Java 8 的局部变量泛型投影 SHALL 只消费该方法同次 `Code` 读取的完整 `LocalVariableTypeTable` 事实，按 slot、名称、BCI 范围与 `LocalVariableTable`、真实局部身份及物理擦除相符后，在语句构建前一次决定类型。只有赋值来源与 Java 泛型构造合法性也获证，完整方法源码才可输出参数化局部与菱形构造。无 debug、事实冲突或证明停止时 MUST 保留现有擦除/强转或既有拒绝，不得凭 `new` 指令猜类型参数。

#### Scenario: Exact Map and HashMap local with debug generic signature
- **WHEN** 唯一 LVTT 与 LVT 均指向 slot 1、名称 `map`、读取范围 `[8,20)`，LVTT 精确为 `Map<String,String>` 且擦除为 `Map`；同轮完整 Code/SSA 证明唯一 `astore_1@7` 紧邻范围起点、其 SSA 输入来自唯一无参 `HashMap` 分配并写入该局部，范围内恰有一次读取，Java 8 平台关系证明 `HashMap<K,V>` 可赋给 `Map<K,V>`
- **THEN** 完整源码 SHALL 在该局部写出 `Map<String,String>` 和 `new HashMap<>()`；原、JADX、Jarde 完整类型与同一 consumer SHALL Java 8 重编并经 `-Xverify:all` 运行一致

#### Scenario: No debug metadata states no type arguments
- **WHEN** 对应 `-g:none` class 没有 LVTT 泛型事实，物理指令与 `-g` 的执行语义相同
- **THEN** 系统 MUST NOT 补写 `String,String` 或菱形类型实参；现有 raw 构造与显式 cast SHALL 保持可编译、可验证和执行等价

#### Scenario: Debug identity, body, or budget proof fails
- **WHEN** LVTT 缺失、重复、损坏、与 LVT 的 slot/范围/名称或擦除不符，局部有第二写入/复用、构造来源无法唯一对应，或同轮读取/规划遇到预算耗尽、取消
- **THEN** 泛型局部与菱形投影 MUST 整体拒绝或传播停止，MUST NOT 发布半个声明或修改不相关 `new`；物理方法、原始 Code 与可得的来源信息 SHALL 保留
