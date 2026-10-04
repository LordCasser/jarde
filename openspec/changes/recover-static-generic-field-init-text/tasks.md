## 1. 取证与基线（root 已完成大半）

- [x] 1.0 损坏样例、判别（静态 only/菱形显式同坏/实例字段不受影响）、jadx 有解、关联拒绝 `field_generic_body_unproved`——全部实测并归档。（root 已完成）
- [ ] 1.1 **定位内联拼接点**：读 `src/class_source.rs` 的静态字段初始化呈现分支（`field_generic_body_unproved` 拒绝后的回退），找到产出 `new MN$Hold` + `ava.lang.Object)` 的拼接代码行，说明丢 `(` 与类型实参的机制；转录存证据目录。
- [ ] 1.2 重验基线：主线二进制渲染 `MN`（两条腿）确认损坏文本仍现、`javac` exit 1；`RG`（嵌套形）同验。
- [ ] 1.3 冻结 fixture：`MN`（javac23 `--release 8` 腿）+ `MN8`（真 javac 8 腿，Corretto 1.8.0_432）入 `tests/fixtures/`，README 记编译命令与 SHA。

## 2. 修复

- [ ] 2.1 按定位结果修复：内联路径产出**路径 A 裸类型正确形**（`new Hold((java.lang.Object) "a")` 同构 f3），或该分支改为响亮引注；**禁止任何损坏中间态**。
- [ ] 2.2 若选路径 B（复用 f3 构造器赋值通道），先验证静态字段该通道存在且语义等价（`<clinit>` 呈现），报告说明取舍。
- [ ] 2.3 不改 `field_generic_body_unproved` 拒绝注释本身（投影域后续片）。

## 3. 验证与验收

- [ ] 3.1 主锚：`MN`/`MN8` 双腿渲染无损坏文本（`grep -c 'Holdava'` 输出 0）、`javac --release 8` exit 0、`main` 输出与原 class 一致（`a b 5`）。
- [ ] 3.2 零回退：`MN.f3` 逐字节不变；corpus 双腿扫描差异类仅为静态泛型字段初始化形（如实记录数量）；`RG` 宿主的非静态字段不受影响。
- [ ] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy（ci.yml 46-76 逐字）+ openspec strict + `git diff --check` + 再生 fingerprint。
- [ ] 3.4 root 独立复核：拼接 bug 机制转录、修复形态合规（无损坏中间态）、主锚/零回退实测；关闭 summary.md 呈现缺陷登记行。（留 root）
