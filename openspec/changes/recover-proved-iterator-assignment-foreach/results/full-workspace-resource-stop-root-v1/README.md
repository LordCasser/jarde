# 全工作区资源中止：不是全量通过

root 从零运行 96 批，前 44 批通过，第 45 批因机器可用空间跌破 5 GiB 被进程组守卫中止。execution.json 保留原始 failed 状态；finally clean 的 pre-command 空间检查拒绝启动清理，原失败不改写。随后 root 独立 cargo clean 实际 exit 0，1570 files / 344.0 MiB 被删除，target 已移除。独立清理 raw/实际调用单列，不能替代失败 finally。后续必须从零重跑，禁止拼接通过批次。

同次退役旧重复 CLI v3/v4/v5 无损 gzip，解压 SHA 核准后删除旧未压缩副本，净回收 145652418 bytes；当前候选、前片最终 CLI 和原始对照证据保留。空间波动原因未证明。构建门槛仍 5 GiB / target 1 GiB。
