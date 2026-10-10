# 乘法 CI 独立验收脚本 v6

v6 对固定 expected 的表示做最小修正：execution JSON 将 tuple 序列序列化为 list 序列，verifier 比较前把其每一行规范为 tuple；`null` 仍保持 `None`。stdout 实际计数解析、`actual` 对照、非空及 `failed == 0` 要求都不变，runner v5 不变。

v6 只准备、未执行；结果路径为 `ci-product-v1/acceptance-field-multiply-v6.json`。只读 inspection 当时的 v5 execution JSON 显示状态为 `running`、记录了前 5 条命令；其中固定 expected 是 JSON list，actual 的汇总也为 list。本说明不把这份中途状态当作完整 build 验收。
