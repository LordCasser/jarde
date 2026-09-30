## 1. 基线与负例

- [x] 1.1 重放固定 A1/A2：核 fixture SHA、A2 不可编译输出与 A1 对照；确认槽 2 的两段定义/读取集事实（SSA def 归属）并记录。（fixture SHA 逐字一致；基线恢复文本与既有记录逐字重放；A2 `javac --release 8` 两处 `boolean[]无法转换为int[]/boolean`；槽 2 段事实与呈现形态记录见 [retype-diagnosis.md](../../evidence/java-syntax-2026-10-01/em17-slot-reuse-patrol/retype-diagnosis.md)。）
- [x] 1.2 构造并冻结至少四个 verifier 有效变体：三段类型交替（int[]/boolean[]/Object[]）、同类型多定义（不分段，逐字不变）、交叠读取真别名（维持现行为）、段内 phi 合流（维持）；记录实现前后输出。（V1–V4 冻结于 `tests/fixtures/p3-array-slot-retype-locals/`；前后输出与 SHA 见 [variants/](../../evidence/java-syntax-2026-10-01/em17-slot-reuse-patrol/variants/) 与 [retype-diagnosis.md](../../evidence/java-syntax-2026-10-01/em17-slot-reuse-patrol/retype-diagnosis.md) 第 4 节。）

## 2. 分段呈现

- [x] 2.1 呈现层按定义分段（design 决策 1–2）：段边界、读取集不相交判据、各段自有声明（或新名，编译事实裁决）；A2 整类 `javac --release 8` 通过且行为与原 class 一致；A1 与 1.2 边界变体逐字/正确。（实现 = `reuse.rs` 第三条窄拆分证明 `array_retype_split`（判据：非参数/守卫头/无 LVT、各写均经 array 通道、≥2 拼写不同、每读值 SSA def 唯一归属、def 值无跨下写存活）；同作用域重复声明经编译探针裁定不合法，段二以既有命名碰撞路径新名 `local2_2` 就地声明；A2 重编 `2,3,4,true` 与原 class 一致；A1/V2/V3/V4 逐字不变。）
- [x] 2.2 回归：既有数组证书（嵌套初始化器、布尔数组、窄存、部分分配）、`p3_java_recovery`、17b/typing 家族全绿；预算/取消不变。（全仓 workspace 全绿；预算语义与取消路径不变——P5 账本按仓库既有重钉机制记录新证明的 `AnalysisSteps` 计费（flat-mixed +4、语料双臂 +23，其余维度零移动），预算拒绝/取消用例全绿。）

## 3. 验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿、fmt、CI 完整 30 项 allowlist clippy（避免 `iter().copied().collect()` 类 CI-only lint）、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [x] 3.2 A2 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 正常路径（含副作用计数）逐路径一致；记录输出 SHA。（见 [results/retype-threeway.md](../../evidence/java-syntax-2026-10-01/em17-slot-reuse-patrol/results/retype-threeway.md)；V3/V4 负例维持既有呈现、Jarde 腿按既有拒绝记录，JADX V4 自身输出不可编译照录。）
- [x] 3.3 root 独立复分段判据、呈现形态与三方行为，更新 EM-17/18 账本与巡查记录。（root 于合并主线 bd535c63 复核：A2 分段 `boolean[] local2_2 = new boolean[3];` 就地声明+段二换名、整类重编行为逐字一致（`2,3,4,true`）、A1/同型/真别名负例逐字不变、全仓 2734/0、fmt/openspec 229/229。落点 `reuse.rs` 第三条窄拆分证明（既有 `Plan::variable_at` 扩展点）复核认可为正确所有者。裁决：int[]/int[][] 维度差异复用按完整拼写差异分段**保留**——与元素类型差异同为单声明不可编译问题，宽判据正确。V3/V4 真别名不可编译呈现维持，属后续片域。）
