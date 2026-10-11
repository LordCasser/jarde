# Compound Builder Stop tests — private compile-fix v2

基于 root 格式化后的 `build.rs`（SHA-256 `97b018e87c6a6c86d8bdb4d1ac3d717bee488d7f9ea5258f819351779f94de40`）。只修改测试代码：四处 `prepare_compound_condition` 成功断言改为显式处理 `ValueRenderFailure::Refusal/Stop`，不为生产类型增加 `Debug`；三个 emitter 调用复用 `RegionPath::new()` 对应的 `Vec<u32>` 测试路径。增加 proof/emission usage 与 Stop 的 `eprintln!`，供 root 锁定真实阶段和位置。

未改主工作区，未运行 Git/Cargo/JDK，也未编译候选。

- base SHA-256: `97b018e87c6a6c86d8bdb4d1ac3d717bee488d7f9ea5258f819351779f94de40`
- candidate SHA-256: `82757c6ee1f8257507f33c26dc5ed7f965b19b240736cda71b6e02aadd5adf5d`
- patch SHA-256: `8445e0acaae32deb0e20c9edfd21b292fa7e141f32620ac1cc59a60ed7e5e36c`
