## 1. 取证与基线（root 已完成大半）

- [x] 1.0 判别（同构恢复/异构拒/jadx 有解/字节码事实 anewarray Number + Integer/Long 元素）——实测归档。（root 已完成）
- [ ] 1.1 定位拒绝发出处：拒绝文本 "the array initializer element … while the array component" 的代码位置；确认组件类型与元素类型的现有事实载体；转录存证据。
- [ ] 1.2 重验基线：主线二进制渲染 `CT`（`cov` 拒、`up`/`io`/`io2` 恢复）；负例探针（非法赋值方向）现状拒绝记录。
- [ ] 1.3 冻结 fixture：`CT`（javac23 `--release 8` 腿）+ `CT8`（真 javac 8 腿）入 `tests/fixtures/`，README 记编译命令与 SHA。

## 2. 实现

- [ ] 2.1 按 1.1 结论把元素-组件一致性判据改为**单向赋值兼容**（决策 1）：同型或组件为元素超类即接受；降向与无关节仍拒。
- [ ] 2.2 呈现按仓库既有数组初始化约定（决策 3 的 Open Question 2），测试钉死。

## 3. 验证与验收

- [ ] 3.1 主锚：`cov` 恢复；整类渲染 `javac --release 8` exit 0、`main` 输出与原 class 一致（fixture 双腿）。
- [ ] 3.2 零回退：`up`/`io`/`io2` 逐字节不变；负例（非法赋值方向）仍拒；corpus 双腿扫描差异类仅为异构初始化形。
- [ ] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
- [ ] 3.4 root 独立复核：判据方向正确（赋值兼容而非放宽）、零回退实测；关闭 summary.md 登记行。（留 root）
