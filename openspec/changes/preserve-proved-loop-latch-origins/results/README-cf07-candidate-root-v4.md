# CF-07 新候选完整类重放入口

当前脚本 `prepare-cf07-candidate-root-v4.py`，SHA-256 `91426f23deebdb5be099f02f25fa5d2c1f5f321114db4397b3532b299722f823`。复用 SHA 固定的 CF-07 collector：非 main 加载，覆盖输出和新 CLI/meta 后才调用其 main；不重复编译运行流程、不回退旧 CLI。保留双 JDK 原2/JADX4/Jarde4完整类腿，正文不改、fresh classes、empty CP/SP、原始运行输出比较。

root 已全文审查 Luna v2 和 root v3/v4 局部差异。v1 为静态拒绝；v2 只通过 Luna helper 预检，未执行 main。v3 的 root 只读观察校验实际退出1：method_key 返回 tuple，而 javap census 使用字符串键；v4 修正此键、提前 return、指令空格，并将 derived latch 限于以 while 开始、以 } 结束的非空语句跨度。

root 实际 v4 helper 预检退出0，闭合旧118文件/29命令/10腿，真实 BLAKE3 模块可用，原 main/Java/CLI/Cargo 未执行；记录 `cf07-candidate-helper-preflight-root-v4.json`。随后 root 对真实旧基线调用观察校验：准确因 andWhile@15 无 derived while 来源而拒绝，记录 `cf07-candidate-observation-helper-check-root-v1.json`。这只确认工具预检及旧输入拒绝，不是新候选接受。

资源满足并冻结新 CLI/meta 后，root 用实际冻结 SHA 执行：

```sh
uv run --offline --with blake3 python -B openspec/changes/preserve-proved-loop-latch-origins/results/prepare-cf07-candidate-root-v4.py \
  --cli-sha256 <actual-frozen-cli-sha256> \
  --metadata-sha256 <actual-frozen-metadata-sha256>
```

新输出 `cf07-candidate-root-v1` 尚不存在。脚本绑定 validation-build-root-v3/execution.json、源码/测试/canonical pins，收集全部方法来源变化并保持旧正文与既有来源多重集；两 JDK 均要求 andWhile@15 和 counted@30 derived while 跨度，default/all 正文及 map 恒同。counted 原源码 for 在旧 Jarde 中实际为 while，本片不改这种正文。counted@20 与 lastIndexOf@25 只核物理锚点并报告，不冒称修复。新重放之后仍需 root 独立验收；tasks 保持2/6。
