# 混合族静态子集折叠实现证据（change `recover-inner-class-static-mixed-folding`）

基线：worktree 主线 `438e1a2d`（实现前重放）。分支取证与变体冻结见
[`branch-forensics-mix.md`](branch-forensics-mix.md)；本文件记录实现、验收与登记缺口。

## 1. 实现（task 2.1）

- **提取落点**（`src/member_inner.rs::scan_family_root`）：静态行收集本就与候选解耦（循环内
  `static_members.push(..); continue;`）；解耦点在返回序——新增
  `FamilyRootScan::StaticMembersWithInstance { statics, candidate }`，`≥2 静态 + 非静态候选` 的
  整族拒绝门（"multiple direct static member rows are outside the one-child family subset"）
  随之删除；`declaration-only static abstract` 门收窄为 `candidate.is_none()`（它只对"单静态经
  `Candidate` 选择"的形态有意义，混合族的静态行走折叠通道后不再经过它）。
- **消费缝**（`src/facade.rs` 类装配入口）：混合 scan 在缝上分解——非静态候选与 `Candidate`
  共享同一窄通道分支（or-pattern），静态子集 `statics` 走与纯静态族完全相同的
  `prepare_class_source_static_member_fold` → `project_class_source_static_member_fold` 通道
  （投影四件套零改动）。投影边界=外围+静态子文本；子对非折叠成员（Inner）的引用保持池拼写
  （重写目标列表只含静态子集）；锚定沿用上片"全部覆盖 source-map 段 + 指令 CP owner/catch
  handler"泛化，无法锚定即拒绝（保守）。
- **声明规则**：混合族折叠拒绝时不声明——保持窄通道已发布的 prepared 家族与其拒绝原因（与
  上片"拒绝的 fallback 不动窄通道家族"同规则）；折叠恰在投影成功时占用报告（text +
  member_family=PreparedStatic）。纯静态族照旧"scan 行总声明折叠结果"。

## 2. 折叠输出（`results/`，SHA 见 [`results/sha256-mix.txt`](results/sha256-mix.txt)）

- **N1**（`results/N1-mix-after.jarde.java`）：`static class Stat extends java.lang.Object` 嵌套
  声明 + 构造名 `Stat()`；域内静态引用源码拼写 `new Stat().use(new N1())`（代码行无 `N1$Stat`
  池拼写）；`member_family.state="prepared_static"`、`members=[Stat(0x0008)]`、
  `projection="projected"`。非折叠 Inner 引用保持池拼写（`N1$Inner make(int arg1)`、
  `new N1$Inner(this, arg1)`、`new N1$Inner(new N1(), 3).total()`）——与分离平铺同口径。
- **MV3**：`static class StatA` + `static class StatB extends StatA`（兄弟继承域内源码拼写），
  Inner 分离。
- **MV2**：`static class Stat` + `new Stat().m()`，Inner 分离。
- **分离单元逐字节不变**：`N1$Inner`、`N1$Stat`、`MV1`、`MV1$Inner`、`MV2$Inner`、`MV3$Inner`
  的 before/after diff 为空。

## 3. 重编与三方对照（task 3.2）

家族类集口径（design Non-Goal："分离语境池拼写保持——编译需家族类集"）：折叠根单元 + 分离
Inner 单元一起 `javac --release 8`（分离单元声明顶层 `X$Inner` 类，使池拼写可解析；二进制
classpath 不能解析嵌套类池拼写——分离单元必须在编译集里），原 jar 留在 classpath，运行
`java -Xverify:all`：

| 家族 | 原 class | 固定 JADX（`jadx/`）重编 | Jarde 家族集重编 |
| --- | --- | --- | --- |
| MV3（两静态+一非静态） | `6`/`6` | `6`/`6` | `6`/`6` |
| MV2（单静态+单非静态） | `11`/`8` | `11`/`8` | `11`/`8` |
| MV1（纯非静态，负例） | `9` | `9` | `9`（不折叠，分离集照旧） |
| N1 | `10`/`7`/`13` | `10`/`7`/`13` | 见登记缺口 |

**N1 登记缺口（非本片回归）**：`Stat.use` 的 `outer.new Inner(9)`——参数限定符 +
`Objects.requireNonNull` 空检查舞蹈的限定 new 形状——恢复层无证明（上片与主线均如此；根
`main` 里的 `new N1().new Inner(3)` 因限定符是新分配、无空检查而恢复）。折叠按合同原样携带
未恢复体（引注不隐藏）。家族集重编恰好 1 个错误（`N1.java:77 缺少返回语句`）；实现前分离
家族集（三单元）同样恰好 1 个错误（`Stat_unit.java:32`，同一 `use`）——折叠不新增编译缺口、
少一个单元。以第二片透明形替换该唯一缺口后家族集重编 `-Xverify:all` 运行 `10`/`7`/`13` 与
原 class 逐字一致。限定 new 恢复属第二片 `recover-inner-class-instance-folding`。

## 4. 门禁（task 3.1）

- `cargo test --workspace --tests --locked --no-fail-fast`：**2875 通过 / 0 失败**（289 个测试
  二进制全部 ok；含上片 `member_class_static_folding` 7/7、`member_family_identity` 20/20、
  `nested_type_source_spelling` 3/3、enum/注解折叠通道全量）。
- `cargo fmt --all -- --check`：通过。
- clippy：CI 实有 29 项 `-A` 清单 + `-D warnings` 通过。
- `openspec validate --all --strict`：252/252。
- 预算/取消原子性：`output_bytes-1` 与先取消 token 均不发布任何混合折叠（测试
  `budget_and_cancellation_stop_the_mixed_fold_atomically`）。
- 磁盘纪律：每轮构建/测试前 `df -h /` 检查（34-54Gi 区间，未触 12Gi 线）。

## 5. corpus 双腿扫描（task 3.1/3.2 补充）

见 [`corpus-scan-mix.txt`](corpus-scan-mix.txt)：主线基线 `438e1a2d`（临时 worktree 构建，已
清理）与本片二进制逐 jar 逐类 `class-source --format text` diff。

## 6. 登记缺口（第二片范围）

- **非静态成员折叠**（`class Inner` 嵌套声明）与 this$0 消参、限定 new 呈现、access$000 消桥
  ——全部第二片；本片 Stat 体对非折叠 Inner 的引用保持池拼写（分离语境同口径）。
- **限定 new 恢复**（参数限定符 + requireNonNull 形状，N1 `Stat.use`）：恢复层无证明，折叠
  原样携带（见 §3）；实现前分离呈现同样没有。
- 混合族中**窄通道投影成功**的非静态成员（capture/calls 全证明形态）：其根文本已被窄通道
  占用时折叠按上片优先级规则拒绝（"the fold replaces only a text none of them claimed"），
  静态子集保持分离——该形态的合并装配留第二片（族装配序依赖第一片落地）。
- 孙代、泛型子 `Signature`、根/子初始化投影复现失败拒绝折叠：沿用上片边界，未动。
