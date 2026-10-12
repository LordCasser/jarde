# Clippy 修正后的重点测试与 CLI 冻结

root 使用 frozen focused runner v7 实际执行 focused-root-v12，八条命令全部 0。公开 API 7/0/1、Java 库 375/0/0（含四条 census 回归）、reader census 1/0/0、P5 指纹 5/0/1。ignored 诊断保持 ignored，不计为通过。4822 文件来源闭包前后相同，默认 stack、5GiB/target1GiB/一秒守卫，最后 cargo clean 0 且 target 不存在。

冻结 CLI root-v4 为 61920032 字节、0555、SHA d3d15d3dd875bc79410298e3ed1e56010fc39c1849e97dc893cadb49d8c65e6f。root 已纯只读执行矩阵 checker 的 focused 完整预检，原始具名结果与 current source/CLI 准入通过；这不是整类矩阵验收。此前 v11/CLI root-v3 的通过属于旧来源，不再借给新源码。完整 workspace-root-v3 当时仍在执行，任务 3.1–3.4 未关闭。
