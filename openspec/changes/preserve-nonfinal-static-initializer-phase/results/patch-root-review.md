Luna v1 的生产 hunk 漏掉既有 has_unknown_static_field 拒绝分支，git apply --check 失败，未写产品。root v2 从当前文件生成准确上下文，只修改 final flags 条件；既有未知外部静态字段检查完整保留。测试 hunk 独立 check 成功，测试内容经 root 审查沿用。原失败 patch 保留。
