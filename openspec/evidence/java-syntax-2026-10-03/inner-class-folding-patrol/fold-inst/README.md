# 非静态成员类折叠实现证据（change `recover-inner-class-instance-folding`）

基线：worktree 主线 `c7b7b767`（实现前重放，debug 构建）。取证与变体冻结见
[`branch-forensics-inst.md`](branch-forensics-inst.md)；本文件记录实现、验收与登记缺口。

## 1. 实现（task 2.1/2.2）

- **折叠（2.1）**：facade 类装配缝在窄通道 `Prepared` 且未 Projected 的非静态候选上发起
  联合实例折叠：`prepare_class_source_instance_member_fold` 证明一般化捕获
  （`prove_family_instance_capture`）、装配 member target + captured outer reads、证明读桥
  （`prove_fold_access_bridges`：ACC_SYNTHETIC 静态 + `access$NNN` 名模式 + 恰三条指令的
  单读直通体）、并经 xref 全域 census 关闭捕获字段/实例构造/桥调用的全部物理消费者
  （`prove_fold_external_use_census`；桥的族外消费者只失去隐藏、声明保留）。
  `project_class_source_member_fold` 复用静态折叠四件套装配：受影响家族方法以折叠 feeds
  重跑恢复（门：Complete + Java 表示 + 全结构化 + 零 fallback——**重跑不得比物理呈现降级**），
  桥调用位重写为限定字段读，捕获写语句删除，嵌套声明经 `elision` 消参消字段。
  成员序 = 根 InnerClasses 行序；族状态 `PreparedFold`；拒绝时混合族回退静态子集折叠、
  纯非静态族零发布（窄通道家族与拒绝原因原样保留）。
- **消隐（2.2）**：折叠作用域内 ctor 首参消参（`HiddenConstructorParameter`）、捕获字段行
  （`HiddenCaptureField`）、捕获写（`HiddenCaptureWrite`）、读桥（`HiddenAccessBridge`）及
  其调用位（`AccessBridgeCall`）全部带 derived 锚；**分离呈现完全不动**（见 §4 逐字节对照）。
- **预算/取消原子性**：折叠只在投影成功时占用报告文本与 member_family；`output_bytes-1`
  与先取消 token 均不发布任何 `PreparedFold`（测试
  `budget_and_cancellation_stop_the_mixed_fold_atomically` 沿用上片语义覆盖联合路径）。

## 2. 折叠输出（`results/`，SHA 见 [`results/sha256-inst.txt`](results/sha256-inst.txt)）

- **N1**（`results/N1-inst-after.jarde.java`）：`class Inner` 与 `static class Stat` 同一嵌套
  装配；`Inner(int arg2)`（this$0 消参消字段消写）；三种构造语境
  `new Inner(arg1)` / `arg1.new Inner(9)` / `new N1().new Inner(3)`；读桥消隐、
  `return this.tag + N1.this.base;`；`access$000` 声明消失。
- **IV1**：双读桥全部消隐重写（`IV1.this.base + IV1.this.bonus`）。
- **IV2**：读桥消隐重写；**写桥 access$002 保留**（调用位 `IV2.access$002(IV2.this,
  IV2.this.count + 1)` 原样）——写形登记（§3）。
- **IV3**：**不折叠**——链式孙代构造的重跑降级（源拼写 `IV3.B` vs 池拼写 `IV3$B` 的参数
  转换无证据）触发保守门；根单元与主线 `c7b7b767` 输出逐字节一致。
- **IV4**：源码声明的非 synthetic `access$000` **保留**（ACC_SYNTHETIC 门生效），折叠照常。
- **MV1/MV2/MV3 与 FV4:VInner**：纯非静态族（MV1/VInner）与混合族（MV2/MV3/N1）的非静态
  候选全部并入折叠。

## 3. 重编与三方对照（task 3.2）

家族集口径同 [fold-mix](../fold-mix/README.md)（折叠根单元 + 仍被池拼写引用的分离兄弟单元；
本片后 N1/IV1/IV4/VInner/MV* 的家族集都坍缩为单一折叠单元）：

| 家族 | 原 class | 固定 JADX（`jadx/`，dev）重编 | Jarde 折叠单元重编 |
| --- | --- | --- | --- |
| N1 | `10`/`7`/`13` | `10`/`7`/`13` | `10`/`7`/`13` |
| IV1（双读桥） | `12` | `12` | `12` |
| IV4（非 synthetic 桥） | `104` | `104` | `104` |
| VInner（FV4 纯非静态） | `4` | —（静态片 fixture） | `4` |
| IV2（写桥） | `11` | `11`（JADX 将读写桥都内联为 `count++`） | 编译受阻：`access$002` 体未恢复（裸 putfield 语句缺口，**两腿同错**：主线分离家族集同样 2 错误——非本片回归；写形登记） |
| IV3（孙代链） | `6` | `6`（JADX 恢复链 `iv3.new B().new C().c()`） | 不折叠；主线分离集同样编译通过但运行无输出（链构造既有缺口，两腿一致） |

## 4. 门禁（task 3.1）

- `cargo test --workspace --tests --locked --no-fail-fast`：289 个测试二进制，288 个一次
  通过；`p4_plugins::the_plugin_plane_leaves_the_structural_planes_own_answer_untouched`
  在整仓运行中失败、单测复跑 4 次全绿（handoff.md 已登记 flake 家族，重跑判定非回归）。
  计 **2875 通过 / 0 失败 / 46 ignored**（与主线基线 2875+ 持平：本片重写既有测试、不净增）。
- `cargo fmt --all -- --check`：通过。
- clippy：CI 实有 29 项 `-A` 清单（`.github/workflows/ci.yml`）通过，零残留警告。
- `openspec validate --all --strict`：253/253。
- 磁盘纪律：每轮构建/测试前 `df -h /` 检查（15-50Gi 区间，全量测试后已 `cargo clean`
  回收 35.2GiB）。

## 5. corpus 双腿扫描（task 3.1/3.2 补充）

见 [`corpus-scan-inst.txt`](corpus-scan-inst.txt)：主线基线 `c7b7b767` 与本片二进制逐 jar
逐类 `class-source --format text` 的 SHA256 对比（manifest：
[`results/corpus-inst-before.sha`](results/corpus-inst-before.sha) /
[`results/corpus-inst-after.sha`](results/corpus-inst-after.sha)，扫描脚本 `corpus_leg.py`）。

## 6. 登记缺口

- **孙代与链式限定 new**（IV3）：member 呈现类型（源拼写）与跨代 ctor 描述符（池拼写）之间
  的参数引用转换无证据；折叠保守拒绝。需要独立的转换证明切片（或在 member target 上携带
  池拼写转换证据）后，`a.new B().new C()` 才能落形。
- **access$ 写桥**（IV2）：写形（含复合赋值的返回新值形 access$002）不隐藏、调用位不重写
  （design Non-Goal，验一形登记）；其裸 putfield 语句体恢复是既有恢复层缺口（两腿同错）。
- **窄通道投影成功的非静态成员**（上片登记口径不变）：根文本被窄通道占用时折叠仍拒绝。
- 匿名类内联、局部类、接口默认方法捕获、孙代泛型 `Signature`：Non-Goal，未动。
