# 主线交付检查点

root 实际执行只读审计，检查点 `6bfc44833580b3068a0dab6144f59d5c5bb44691` accepted_clean_main；产品 `7e93eb30647338dd8a6ddfc9994392f23457aa3e` 是其祖先。HEAD/main/origin/main/远端真实 main 恒同，local/remote 仅 main；15 个 checkout 中 14 个辅助已 detached，全部干净、主线祖先、无 target/fuzz target。4854 个来源逐字同 focused / product / docs / live，自身 CI 的 v4 接受记录被精确 path/SHA 绑定。

`execution.json` 是实际审计输出，`audit-clean-main.py` 是执行工具，逐文件 SHA/长度见 pins.json。工具不执行 Cargo。此检查点只证明该 HEAD，不证明之后的关闭文档提交；root 随后对实际最终 HEAD 在 `/private/tmp/jarde-loop-field-clean-main-final-root-v1/execution.json` 作外置审计，避免把未来提交的证明写回自身。
