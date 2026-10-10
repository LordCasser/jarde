# 乘法验证 build runner v4

v4 是 v3 的窄版本更新：使用全新的 `validation-build-root-v4` 输出目录和 `int-field-multiply-validation-build-root-v4` schema。9 条命令、source base、磁盘守卫、source pin、CLI 与 metadata 路径均维持原值。runner 会在 execution JSON 中记录自身路径和 SHA-256。

v3 在预检阶段因可用空间仅约 17.7 GiB 而停止，保留 `validation-build-root-v3` 原记录且未开始编译。v4 不触碰该失败记录，也不会覆盖已有 v4 输出、CLI 或 metadata。此脚本只准备，尚未执行。
