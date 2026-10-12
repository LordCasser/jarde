# 最终 Clippy 失败

root 实际执行 final-validation-root-v1，fmt 通过，当前 CI Clippy 因两处切片 `.iter().copied().collect::<Vec<_>>()` 触发 `iter_cloned_collect` 退出 101。strict 和 CLI build 尚未执行；finally cargo clean 通过，target 不存在，源码闭包 4822 未在执行中变化。原失败收据和原始流完整归档，不改写为通过。

root 随后只将这两处复制改成 `.to_vec()`，计费、顺序和比较不变。修正后源码需要新的 focused、全工作区、CLI、整类对照和最终检查；旧 run 的来源不能冒充当前产品。
