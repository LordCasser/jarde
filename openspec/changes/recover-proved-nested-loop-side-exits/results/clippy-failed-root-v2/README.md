# Clippy 原失败与等价单行修正

真实fmt0/Clippy101/clean0，首个lint unnecessary_first_then_check 指 region.rs 新消费函数的 prefix.first().is_none()。prefix 是 Region::Straight.blocks 的 Vec，Luna私稿/root逐字节核唯一替换为prefix.is_empty()，空/非空判断语义等价，无lint allow/生产机制变化。原raw、完整source pins和private base/candidate/patch/review保留。此前fresh workspace v2真实3455/0/97已通过，但整个closure因单行变化不同，不能宣称最终源已有该全量执行；后续先fresh lint，最终新CLI和fresh检查另验。

Root首次apply脚本误按startswith取base文件，assert在共享写入前阻断；随后明确选region.rs.candidate并核上述唯一字节替换后才应用。首次lint-v3未创建因此命令exit2，无Cargo执行，不当真实新lint失败。
