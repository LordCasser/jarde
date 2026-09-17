## 1. 修正 P1 验证工具

- [x] 1.1 实现共享五请求驱动并让真实 query fuzz target 使用它；合法 CLASS/JAR 和受损种子测试观察实际路由及结果，临时恢复旧选择器须使回归失败
- [x] 1.2 CI 显式审计根/CLI 与 fuzz workspace，CLI 既有 path 依赖补精确版本且保持 wildcard deny，NCSA 收窄为指定 libfuzzer-sys 例外；两个图 cargo-deny 四段通过，临时拒绝 fuzz-only 依赖及移除许可例外均须按预期失败

## 2. 验证与交接

- [x] 2.1 更新 fuzz README；运行 fuzz workspace fmt/test/clippy、根必要回归、本地两个 target 各 60 秒单 worker smoke，记录环境/peak RSS，保持 512 MB 限额及锁文件不变
- [x] 2.2 完成只读复核、OpenSpec strict 与 diff 检查，记录结果及远端 CI 是否实际运行；将 p2-jvm-ir 的 V1/V2 状态指向本变更，确认 P2 无新增代码且任务全部未勾选
