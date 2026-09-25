# Root 独立复核：多方法变量和简单多界子切片（2026-09-24）

Root 审读当前 `project_method_signature`、`generic_method_declaration`、`simple_generic_class_name` 和新定向测试。新增准入仍以同一物理成员的唯一 `Signature`、reader 逐位置擦除及同轮 Program/SSA 参数来源为前提；多个方法局部类型变量、简单 class/interface 界及非参数化普通参数可完整拼写。返回值仍须来自对应变量的参数，或布尔参数在两个该变量参数间选择。`^T` throws、未知 flags、参数化/嵌套界、类型注解、varargs、正文赋值/调用及同类调用不在已证子集，整项拒绝并保留 descriptor 声明。没有改变独立方法报告。

Root 使用独立 `CARGO_TARGET_DIR=/tmp/jarde-generic-root-accept-target`、`CARGO_INCREMENTAL=0` 执行 `cargo test -p jarde --test generic_method_projection --test generic_method_budget --test class_source --locked`：分别 9/9、4/4、47/47 通过。审读新增完整类测试可确认，它用 `javac --release 8 -g:none` 同时编译原/恢复类及独立反射调用方，`java -Xverify:all` 执行两者，并逐字比较普通调用值、两个变量的名称/界、四个泛型参数及泛型返回；不兼容正文和泛型 throws 拒绝也覆盖。`cargo fmt --all -- --check`、`git diff --check` 及 `openspec validate recover-generic-method-signatures --strict` 均通过。

本次只验收上述多变量子切片。任务 2.3/3.1 要求赋值、表达式合流、正文与同类调用绑定的源级证明；这些仍未实现，两个任务维持未勾选。Root 已执行 `cargo clean --target-dir /tmp/jarde-generic-root-accept-target`，移除 878 个文件、599.0 MiB；该临时构建不作为持久产物。
