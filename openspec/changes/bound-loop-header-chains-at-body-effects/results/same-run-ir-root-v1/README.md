## Accepted facts

root实际运行一次完整Reader/SSA/Region探针，1 passed / 0 failed；源码精确恢复、临时模块移除、target absent，guard无停止。debug/no-debug均有自然header7、body14/50、latch63与最终exit72；Map.get/字符串构造在14，null分支保护50的字段读取，两项更新只在63执行。实际首gap为后续14的StatementFree/15，单表头ForHeader路径尚未到达；辅助直接检查None仅是辅助事实。

`capture.tar.gz`内含逐文件bytes/mode/SHA清单，root已逐成员复开。保留v1/v2未运行准备、root退回意见及对预建快照目录的事实修正、冻结v3、实际guard和runner全部原始流及清理收据、只读架构笔记和归档脚本。`root-review.json`给出归档身份；不包含CLI或工具链payload，不代表产品实现已验收。

完整原件/JADX/Jarde基线另见`openspec/evidence/java-syntax-2026-10-19/cf09-testbreakincomplexif-original-root-v1/README.md`，本轮root重新核验该归档全部399个payload；没有重跑已接受基线。
