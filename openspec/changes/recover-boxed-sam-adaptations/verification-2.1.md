# Root 验收：两段 SAM 类型适配证明

`lambda.rs` 保留 capture 的 frame/site/implementation 精确同型门，仅把 SAM 参数与返回按擦除类型、instantiated 类型和实现描述符逐边核对。八种基本类型只与各自包装类配对；`Object` 检查/上溯沿用既有范围。方法引用的参数适配仍转为箭头候选，绑定接收者的创建时 null 检查不延后。此步只交付 2.1 的类型证明，尚未证明数组合成 helper，也未宣称冻结 `BoxedSamProbe.main` 完整恢复。

Root 审读 `Plan`、捕获门、两向转换和 `Builder` 现有 cast 消费后，在同一主线重新运行：`cargo test -p jarde-java --lib --locked` 189/189；`p3_immediate_functional_receivers` 2/2；`p3_lambda_adaptation` 排除两项已知预算测试后 7/7，另有 2 项需要 JDK 的旧 ignored。当前完整 `p3_lambda_adaptation` 的 `adaptation_budget_stops_and_precancellation_do_not_publish_a_partial_body` 与 `captured_adapter_budget_stops_inside_nested_value_rendering` 仍失败；实现代理在干净 `bfb9e849` 隔离 worktree 上复现相同断言，未把它们归因于本项。`class_initializer_candidates` 的两项预算失败也在该干净基线复现，属于独立预算测试债务。

Root 用当前 CLI 对冻结 `BoxedSamProbe.class` 重放：`supplier` 与 `minimumSupplier` 形成方法引用，`function`、`arrayCtor` 的使用点现在有箭头文本，完整类的 `main` 仍引用；该输出可由 `javac --release 8` 编译，但尚不能以此当成整类行为验收。后续 2.2 须从真实合成 Code 证明数组构造器，2.3/3.1 再做完整文本、边界值、null 拆箱和负数组长度的 JVM 对照。
