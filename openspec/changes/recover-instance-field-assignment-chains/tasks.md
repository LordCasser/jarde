# Tasks

> 纪律：门控实验先行（实例形准入单独翻转 CP.inst、CH 静态锚不翻）；行号按锚点名重验（FieldCopies 于 build.rs，锚=结构名）。

- [x] 1.1 复核探针（CP 类重编双腿）+ 定位 Chain proof 的 dup 读取处（为何 `dup_x1` 不入读）；门控实验转录存证据。
      → `results/01-probe-instance-chain.txt`：CP 源重编双腿（两条腿指令序列逐条相同、仅常量池索引不同）、栈几何逐步
      （`[…, B, T] → […, T, B, T]`：拷贝插在 receiver 之下）、SSA 身份表、基线拒绝逐字重放；dup 过滤器定位
      （`prove` 的 `OPCODE_DUP_X1` 臂只走 receiver 副本与条件 receiver 副本，`field_chain_at` 只从 `OPCODE_DUP` 进入）。
      → `results/01-gating.md` + `results/01-gating.sh` + `results/renders/*`：实例形准入单独翻转 CP（inst/pair），
      CH/SC/BF/BG/CA2/CF 逐字节不变，MX/NEG 逐字节不变。
- [x] 1.2 冻结锚与负例双腿：CP（探针源）+ 混合链（静态+实例同方法）；负例=跨对象链（`o1.a = o2.b = 5`）、非 putfield 消费形。
      → `tests/fixtures/recover-instance-field-assignment-chains/{CP,MX,NEG}.java` 双腿冻结（`v8/`、`v8-javac8/`）+ README
      （编译命令、九个 SHA、逐形状字节码与两腿行为）；CP=实例链锚（inst/pair），MX=混合链边界（三种静态/实例同舞），
      NEG=四负例（跨对象链、`(this.b = 5) + 1` 非 putfield 消费、`f()` 源、`h()` receiver）。
- [x] 2.1 实现实例形 Chain（dup_x1 交织 receiver 判据 + 三 aload_0 SSA 同一性）；既有 Chain/Receiver 判据逐字不动。
      → `results/02-implementation.md`：`instance_chain_at`（三写槽位判据）+ `dup_x1_writes` + `reads_this`
      （slot 0 入口值的同一性）+ `FieldCopy::moved`（receiver 渲染）；`prove` 签名增 `has_receiver`（`MethodFacts::has_receiver`）；
      `field_chain_at`/`receiver_copy_at`/`conditional_receiver_copy_at` 判据逐字未动，静态链 saved 机制未动。
- [x] 2.2 对照测试：CP 恢复（重编+`-Xverify:all` 行为一致）；`recover_chained_field_assignment` 套件零回退；负例拒绝逐字。
      → `tests/recover_instance_field_assignment_chains.rs`（facade `Engine` + `ClassSourceRequest` + jar + self-header 断言；
      ignored 回放：两腿 `javac`（`--release 8` 与真 javac 8）编译 + `-Xverify:all` 运行与原类逐行一致；MX/NEG 剥离文本不编译）；
      既有 `recover_chained_field_assignment` 套件绿、锚逐字节不变。
- [x] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。
      → `results/03-corpus-delta.md`：全语料 before/after 差分 moved=2（本片 CP 两条腿）全归类、oracle 腿 3/3 无陈旧期望；
      `results/04-gates.md`：fmt/clippy/workspace 测试/openspec strict/fingerprint 逐条。
- [ ] 3.2 root 独立复核：门控、判据最小性、锚/负例实测、账本（探针边界关闭）。（留 root）
