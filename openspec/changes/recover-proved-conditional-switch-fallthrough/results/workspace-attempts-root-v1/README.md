# 完整 workspace 的未接受尝试

v1 在启动 Rust 前收集 literal include 输入时，把文档中的示例路径当作真实 fixture，preflight 失败；没有执行测试，也没有完整 execution.json。v2 使用实际存在的 literal 输入并重新从头执行，metadata 实测 359 targets、95 batches；前 21 批完整 command records 实际通过，随后 v9 `target_bytes` 在 `is_file()` 与 `stat()` 之间遇到 Cargo 替换 build-script 文件，抛出 FileNotFoundError。第 22 批原始输出虽保留，不当成守卫成功的 command record。最终 status failed、pins_unchanged true，root 随后实际确认没有 Cargo/rustc 遗留进程。

历史 v9 源文件不修改。Luna 和 root 独立准备了单次 stat 的小 scanner，只有 FileNotFoundError 跳过、S_ISREG 计数，其他错误仍显式失败。root 实际使用 change-local `target-size-scan-root-v1.py`，SHA256 `d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7`，记录为独立 adapter pin；5 GiB free、1 GiB target、一秒进程组守卫没有放宽。v3 从头重跑，不能借用 v2 的前 21 批凑成功。

本目录另保留 strict 实际调用，343 passed / 0 failed / exit 0。copy-manifest 逐项核字节与 SHA，Luna scanner 文件只是静态审查输入，不冒充 root 的实际 runner。最新完整通过数只能来自另行完成的 v3 记录。
