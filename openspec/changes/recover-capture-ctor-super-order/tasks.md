## 1. 取证与基线（root 已完成大半）

- [x] 1.0 双形判别、拼接 exit 1、jadx 双括号解、字节码次序机制——实测归档。（root 已完成）
- [ ] 1.1 插桩/读码定位伴生 ctor 渲染的 val$/super 次序逻辑（member_inner.rs vs facade.rs）；转录存证据。
- [ ] 1.2 重验基线：主线二进制渲染 DB（捕获形 exit 1 现状、无捕获健康）；负例探针（super 实参依赖捕获值）现状记录。
- [ ] 1.3 冻结 fixture：DB（javac23 `--release 8` 腿）+ DB8（真 javac 8 腿——验证两版 val$ 前置次序一致）入 `tests/fixtures/`，README 记编译命令与 SHA。

## 2. 实现

- [ ] 2.1 按决策 1 实现重排（pre-super 全捕获赋值 + super 实参无依赖→重排；否则现状）；呈现次序按决策 2。
- [ ] 2.2 无捕获形伴生与宿主调用形零改动。

## 3. 验证与验收

- [ ] 3.1 主锚：DB 双形拼接 `javac --release 8` exit 0、行为 `2/z` 逐行一致（含真 8 腿 fixture）。
- [ ] 3.2 零回退：无捕获形逐字节不变；负例保持现状；corpus 双腿扫描差异类仅为捕获形伴生（记录数量）。
- [ ] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
- [ ] 3.4 root 独立复核：重排条件保守性（数据流证明而非猜测）、零回退实测；关闭 summary.md 登记行。（留 root）
