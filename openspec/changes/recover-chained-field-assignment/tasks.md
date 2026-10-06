## 1. 取证与基线（root 已完成大半）

- [x] 1.0 拒绝点、局部链判别、jadx 拆分解、三形状族定位——实测归档。（root 已完成）
- [x] 1.1 插桩定位发出处；确认与 #8/#9 片的同/异落点（同机制形状分支 vs 独立）；转录存证据。
      → `results/01-gating.md` + `results/01-gating-transcript.txt`：链与复合 RMW 同落 copy 门（`duplicate_expression` +
      `render_instruction` 的 `Duplicate` 臂——与 dup-store/后缀片同一对文本）；String 累积是**另一落点**（concat 走查的
      `jre_concat_interleaved_effect` + `new@1` 的读者门 + `Other` 值渲染，三拒绝级联），但其**副本形状**就是复合 RMW 的
      receiver 副本——判定为"一门两形状"：新证明一处（`FieldCopies`），两个形状；复合 RMW 因此与核心锚同门、一并覆盖，
      不另行移交。更新规则的 `+=`/`-=` 呈现逐字未动。
- [x] 1.2 重验基线：主线二进制渲染 CH（字段链拒、局部链恢复）；负例探针（`a = (b = 5) + 1`）现状拒绝记录；副作用表达式链探针（方法调用右值）基线记录。
      → `results/01-gating.md` 表 + `results/renders/*-before.txt`：CH/SC/BF/BG 拒、局部链恢复、`DV` 仅 add/sub 恢复、
      副作用链探针与三负例探针均拒（逐字记录）。
- [x] 1.3 冻结 fixture：CH（javac23 `--release 8`）+ 真 8 腿入 `tests/fixtures/`，README 记编译命令与 SHA。
      → `tests/fixtures/recover-chained-field-assignment/{CF,NEG}.java` 双腿（`v8/`、`v8-javac8/`）冻结，
      README 记编译命令、六个 SHA、逐形状字节码与两腿行为。

## 2. 实现

- [x] 2.1 按决策 1/2 实现：dup-跨-putfield 的拆分赋值组（常量直书、副作用形 temp）；负例（表达式内消费/混合消费）保持拒绝。
      → `results/02-implementation.md`：`FieldCopies`（`Chain`/`Receiver` 两形状）+ 渲染臂；常量直书
      （`CF.sc = 5; CF.sb = 5; CF.sa = 5;`）、副作用形 temp（`int saved0 = f(); …`，命名 `saved{N}` 与既有绑定机制同源）、
      负例（局部存储消费/表达式内消费/数组存储消费/调用 receiver）逐字仍拒。
- [x] 2.2 与 #8/#9 的形状分派结构按 1.1 结论组织（同机制分支优先）。
      → 同门控单机制：计划在 report 阶段一次证明（与 `CompoundAssignments`/`ArrayInitializers` 同形），
      `duplicate_expression` 首先查询该计划（copy 门的第一分支），`render_instruction` 的 `Duplicate`/`Other` 臂、
      `renders_the_value_it_reads` 同步；concat 走查只按**已证 receiver 副本**放行 dup_x1 与它的读（不泛化 `Field{Read}`）。
      实例 dup_x1 链（`this.a = this.b = this.c = 5`）如实记录为边界（`results/02-probe-instance-chain.txt`，逐字未动）。

## 3. 验证与验收

- [x] 3.1 主锚：chain 恢复、整类 `javac --release 8` exit 0、行为一致（含副作用 temp 形探针）。
      → `results/03-anchors.md` + `tests/recover_chained_field_assignment.rs`（含 ignored 回放）：CH/SC/BF/BG/CA2/CF
      两腿编译运行逐字一致（`-Xverify:all`），`f` vs `f[x][y]` 面关闭；副作用 temp 形由 `CF.call` 钉住。
- [x] 3.2 零回退：局部链与单字段赋值逐字节不变；负例仍拒；corpus 双腿扫描 diff 为空。
      → `results/03-corpus-delta.md` + `results/03-corpus-sweep.sh`：全语料 moved=8 全部归类（本片 fixture 两条腿、
      CH/SC/BF/BG/CA2 五个巡查锚=干净恢复、**BI 单列**为"恢复+新可达的既有 critical 锚 15 面"，
      见 `results/03-finding-bi-anchor15.txt`）；静态 String 复合/复合边界族/局部链/dup-store/后缀族逐字节不变；
      负例逐字仍拒。
- [x] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
      → `results/04-gates.md`。
- [ ] 3.4 root 独立复核：求值一次语义保持、零回退实测、形状分派结构合理；关闭 summary.md 登记行。（留 root）
