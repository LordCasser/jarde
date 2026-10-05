# Tasks

- [ ] 1. 插桩定夺（root 预审计已收窄）：field.rs:786 `stated_type` 与 build.rs:25393 `array_of_value` 的单变量差异复核；定夺接入方式（field plan 已持有 operations——直接调用 vs 闭包/trait 注入避免 field→build 反向依赖）；记录参数数组/newarray 局部链/别名链三类操作数的实测覆盖
- [ ] 2. 实现 aaload 组件类型传播 + 字段身份证明消费（不可证保持现拒）
- [ ] 3. 对照测试：RG/RH/RK/RL/RJ fixtures 双 javac 协议 + 行为一致 + 直接参数零回退 + EM enum 数据点
- [ ] 4. 全门禁 + 分逻辑提交（不 push）
