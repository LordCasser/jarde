## 1. 取证与基线（root 已完成大半）

- [x] 1.0 判别（同型恢复/异构拒/jadx 语句化有解/return 消费形）——实测归档。（root 已完成）
- [ ] 1.1 定位拒绝发出处：拒绝文本 "the two values joined … do not have a conditional Java type" 的代码位置；确认其对消费上下文（return vs 实参）的可见性；转录存证据。
- [ ] 1.2 重验基线：主线二进制渲染 `IF`（`poly` 拒、其余恢复）；实参嵌套异构探针（新造：`foo(c ? Integer.valueOf(1) : "s")`）现状拒绝记录。
- [ ] 1.3 冻结 fixture：`IF`（含 poly）入 `tests/fixtures/`，README 记编译命令与 SHA。

## 2. 实现

- [ ] 2.1 按 1.1 结论实现 return 消费形的语句化拆分（决策 1/2）：可拆则拆、其它消费形保持既有拒绝；无新类型计算。
- [ ] 2.2 拆分呈现按仓库既有 if/else 约定（决策 2 的 Open Question 2），测试钉死具体形。

## 3. 验证与验收

- [ ] 3.1 主锚：`poly` 恢复为 if/return 形；整类渲染 `javac --release 8` exit 0、输出 `4/9/true/2/s` 与原 class 一致。
- [ ] 3.2 零回退：同型三元（`dense`/`chain`/`io2`）逐字节不变；负例（实参嵌套异构）仍拒。
- [ ] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
- [ ] 3.4 root 独立复核：拒绝点定位、拆分判据未扩（仅 return 消费）、零回退实测；关闭 summary.md 登记行。（留 root）
