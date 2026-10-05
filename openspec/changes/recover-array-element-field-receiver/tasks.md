# Tasks

- [x] 1. 插桩定夺（root 预审计已收窄）：field.rs:786 `stated_type` 与 build.rs:25393 `array_of_value` 的单变量差异复核；定夺接入方式（field plan 已持有 operations——直接调用 vs 闭包/trait 注入避免 field→build 反向依赖）；记录参数数组/newarray 局部链/别名链三类操作数的实测覆盖
      → 预审计三条断言逐条复核成立（行号漂移，机制为准）；定夺**闭包参数注入**（`plan`/`verify` 增 `&dyn Fn(ValueId) -> Option<String>`，`report.rs` 用 `build::element_receiver_type` 提供），理由是最小 diff 与依赖方向，见 [instrumentation.md](instrumentation.md)；四类数组来源（参数 / `anewarray` 局部链 / 调用结果 / `[[` 元素链）实测覆盖
- [x] 2. 实现 aaload 组件类型传播 + 字段身份证明消费（不可证保持现拒）
      → `build.rs::element_receiver_type`（复用 `array_of_value` + `array_spelling` 的既有数组读法，输出内部名）+ `field.rs::verify` 的 `stated_type(..).or_else(..)`（帧命名优先，其余判定一字未动）+ `report.rs` 注入点；不可证（分支/条件合并）保持原拒形与原文
- [x] 3. 对照测试：RG/RH/RK/RL/RJ fixtures 双 javac 协议 + 行为一致 + 直接参数零回退 + EM enum 数据点
      → `tests/recover_array_element_field_receiver.rs`（10 非 ignored 逐字钉成员文本与 BCI 锚 + 3 ignored 剥离-编译-运行重放）；fixtures `tests/fixtures/recover-array-element-field-receiver/`（`v8` = javac 23.0.1 `--release 8`、`v8-javac8` = Corretto 1.8.0_432）；RG 判别（改动前 `null/null/null`）、RH `5/xy`、RK `q/q`、RL `s/i`、RM/RJ/RO 零回退、EM 循环体恢复（常量池形债仍不可编译）、RP 不可证仍拒、RN 写侧随同一比较自然恢复（如实报告）
- [x] 4. 全门禁 + 分逻辑提交（不 push）
      → 门禁与提交见 [verification.md](verification.md)
