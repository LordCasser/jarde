## 0. 前置（root 门禁，未满足不得开工）

- [ ] 0.1 **确认 `recover-anonymous-local-decl-site`（环 1）已合入主线**。本片与环 1 改**同一个门** `anonymous_super_return_type_unproved`（环 1 改其站点形与左端重拼，本片改其参数表），串行实施必然 rebase 冲突且验收无法定位失败原因。开工前用 `git log --oneline --grep=local-decl-site` 或查 `openspec/changes/recover-anonymous-local-decl-site/tasks.md` 的 3.3 是否已勾确认。若环 1 尚未合入，**停手报告**，不要先行修改该门。
- [ ] 0.2 重读 [design.md](design.md) 的五条决策与两条 Open Questions，以及 [anonymous-chain-rings-2-3](../../evidence/java-syntax-2026-10-04/anonymous-chain-rings-2-3/README.md)（本片的机制判定：**不需要新机制**，移植接口路径既有先例）。

## 1. 取证与冻结

- [ ] 1.1 重放锚 `tests/fixtures/proved-java-structure/anonymous-super-dispatch/`：记录当前呈现与拒绝码（应为 `anonymous_super_return_type_unproved`）、完整源集 `javac --release 8` 的**当前退出码**（须实测记录，不得沿用旧记载）、原 class `java -Xverify:all` 输出（该 fixture 的 `run.sh` 已有对照流程，README 记载其观察点是"虚调用发生在 `Base` 构造器返回之前且捕获值已可见"）。用 javap 复核 design Context 中 root 实测的每一项事实（根方法 descriptor `(Ljava/lang/String;)LBase;`、flags `ACC_PRIVATE|ACC_STATIC`、分配点 BCI 0、child ctor `putfield val$captured` 先于 `invokespecial Base."<init>":()V`、`Base` 为顶层类）。
- [ ] 1.2 **冻结 `-g:none` 同形对照**（design 决策 2 强制）：以 `javac --release 8 -g:none` 编译同形源码，放独立目录（如 `anonymous-super-dispatch-nodebug/`），**不得覆盖既有冻结 fixture**；确认 `javap -l` 的 `LocalVariableTable` 计数为 0，并记录两腿的原 class 运行输出一致。
- [ ] 1.3 冻结负例：根方法带**多个**参数；分配实参非单一参数槽；捕获参数在根方法内**另有消费**（如同时打印）；根方法为**实例方法**（非 `ACC_STATIC`）；返回类型为父类的**超类型**（须仍按既有码拒绝，证明本片未顺带放宽环 2）。各负例须**响亮失败**（保持物理文本或不可编译），不得静默偏离；须实测 `javac` 退出码与运行输出，**先自检脚手架**（文件名与 public 类名一致、每项独立目录、直接判 javac 退出码而非管道末端——见 handoff "验证脚手架必须先自检"）。

## 2. 实现

- [ ] 2.1 放宽 `project_class_source_anonymous_super` 的根方法门：**仅放宽参数表**，接受 `(P)Lparent;`（`P` 为被证明的单个捕获参数描述符）；**返回部分保持恰等 `Lparent;` 不动**（design 决策 1）。
- [ ] 2.2 把分配点的捕获实参来源扩为**根方法参数**，判据沿用接口路径先例（`facade.rs` 约 3523–3530）的七项合取：descriptor 匹配、`ACC_STATIC`、`scan.complete`、`site.verified`、`site.argument_bcis.len() == 1`、`argument_parameter_slots == [Some(0)]`、`allocation_argument_bcis` 与 AST 对齐、参数名可从 AST 取得（`class_source_single_parameter_name`）。**参数名不得来自 `LocalVariableTable`**。
- [ ] 2.3 捕获读取的词法替换目标改为**根方法参数名**；呈现类型走 `ProvedCapturedParameterRead.parameter_presented`，**不硬编码 `b"D"`**（design 决策 3）。接口路径残留的五处 `b"D"` 特化（约 3455/3490/3771/3800/3896）**不得修改**——属另一片。
- [ ] 2.4 确认划分退化形正确：本片锚的 super 实参集为**空**、全部构造器参数为捕获角色，`partition_anonymous_val_constructor` 须在该形上不误判为无角色或拒绝（design 决策 5）。
- [ ] 2.5 **不改**站点扫描 `class_source_direct_return_new`、**不改** `emit.rs`（design 决策 4：本片锚已是直返形，不动站点形故不继承环 1 的接口路径遏制义务）。若取证发现实现必须动站点形，**停手报告**。
- [ ] 2.6 回答 design 的两条 Open Questions（实例方法形是否拒绝、捕获参数另有消费的形是否可证），按"默认拒绝并登记"处理，除非取证证明可安全放宽——**放宽须停手报 root 裁决，不得自行决定**。

## 3. 验收

- [ ] 3.1 **渲染源集**（root 已实测钉死基线口径，勿混淆两种源集）：用主线二进制渲染 `anonymous-super-dispatch` 后抽取源码区、与 fixture 的 `Base.java` 组成源集，`javac --release 8` 从**当前 exit 1**（`找不到符号`——渲染文本引用 `AnonymousSuperDispatch$1`，非法 Java 标识符）转为 **exit 0**；`java -Xverify:all` 运行输出与原 class 逐行一致（原 class 基线实测为 `observed=captured-value` / `visibleDuringSuper=true`）。注意：**fixture 的原始 `.java` 源集本来就 `javac` exit 0**，故"源集能编译"不是验收信号——必须用**渲染产物**组成的源集。`-g:none` 对照腿（1.2）同样从 exit 1 转 exit 0 且呈现与 `-g` 腿一致（证明不依赖 `LocalVariableTable`）。
- [ ] 3.2 零回退：`recover-anonymous-mixed-super-capture` 的全部正负例（新锚 `anonymous-super-mixed-direct` 须**逐字节相同**、六个 mixed refusals 仍响亮拒绝）、`recover-anonymous-local-decl-site`（环 1）的锚与遏制负例、`recover-proved-anonymous-local-capture`(6/6)、`recover-proved-anonymous-inner-this`(8/8)、`inline-proved-anonymous-super-arguments`(8/8)、`recover-ctor-reorder-dispatch-guard` 三向负例全部逐字通过。
- [ ] 3.3 门禁：`cargo test --workspace --tests --locked --no-fail-fast`（基线数字以开工时主线实测为准，环 1 合入后会高于 296/2937；已知 flake 家族见 handoff.md，单测复跑两轮判定）；`cargo fmt --all -- --check`；clippy **从 `.github/workflows/ci.yml` 46–76 行逐字生成**（含 `--all-features`、29 项 `-A`、`-D warnings`）；`openspec validate --all --strict`；corpus 双腿扫描（差异应仅本形；**出现第 2 个差异类即越界信号，停下报告**）；`git diff --check`。磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，**报告前必 clean**，同一时刻只允许一个 cargo target。
- [ ] 3.4 root 独立复核根方法门放宽的边界（返回部分是否仍恰等）、参数名来源是否真为 AST（用 `-g:none` 腿验证）、划分退化形、接口路径零波及（`b"D"` 五处未改）、三方行为与账本更新（DT-06 匿名父类域 + `present-proved-java-structure` 5.3 剩余范围）。（留 root）
