## Context

基线 `8fcdd66`。复核证据见 p2-jvm-ir/design.md 的 V1/V2；本 change 承接其验证维护，不修改 P1 主规格或 P2 生产代码。

## Goals / Non-Goals

**Goals:** 让合法 CLASS/JAR 经真实 fuzz 入口到达五种固定请求；根与 fuzz 的依赖均受同一 CI 策略审计。

**Non-Goals:** 不扩大 consumer schema，不新增 fuzz 输入格式/自定义 mutator，不修改引擎，不提高 512 MB 限额或改锁文件。

## Decisions

1. 一个共享 query exercise 函数接收整个 artifact，open 一次，再顺序执行 0..QUERY_SHAPES。每次 query 使用现有 limits 的独立预算并即时断言/释放结果；最多一次 open 加五次 query，整体工作量有固定乘数界限。单次 query 的公开错误不阻止其他形态执行。target 直接调用该函数，corpus tests 通过同一路径观察每种请求实际调用及结果，避免测试另一条自建循环。
2. 仍复用全部既有 assert_query_contract；合法种子须得到有内容的预期关系，并报告 Verification/Debug 未实现；损坏种子不能伪称完整。回归测试观察实际访问请求/返回结果，不仅检查常量 QUERY_SHAPES。临时还原首字节选择器，要求新回归失败，再恢复修复。
3. CI 保持一个 supply-chain job，显式对根及独立 fuzz manifest 执行同一 policy；根须包含 CLI workspace member。使用明确 manifest/config/locked 设置。许可例外只覆盖 libfuzzer-sys 0.4.13 的 NCSA，普通许可继续走原 allow，生产依赖边界保留。显式审计 CLI 后暴露其既有 path 依赖没有版本约束，补 `version = "=0.1.0"`，不把 wildcard deny 降为 warn，不修改依赖解析或 lockfile。cargo-deny 的配置和 action 参数按实际版本/官方源码核对。
4. 不增通用审计框架。用临时 policy 拒绝只存在于 fuzz 图的依赖，根检查应通过而 fuzz 检查失败；另临时移除 NCSA 例外应使 fuzz licenses 失败，恢复后通过。临时文件在仓库外，提交锁文件不变。

## Risks / Trade-offs

- [Risk] 每输入执行五次 query 降低吞吐并改变 ASan 内存峰值 → 顺序释放结果，保持小预算，记录相同工具链下 60 秒 query/tree smoke 实测；次数不作为覆盖证明。
- [Risk] macOS 本地通过不代表 Linux action 已运行 → 本地命令与远端 CI 证据分开，不伪报未运行的远端 job。

## Migration Plan

先完成回归与 harness、独立完成 CI/policy，然后合并验证并记录只读复核。只标记已有执行证据的任务。P2 继续保持规划状态；不把 P1 原有缓存/坐标等债务混入本变更。
