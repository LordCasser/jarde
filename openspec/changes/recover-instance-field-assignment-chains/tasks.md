# Tasks

> 纪律：门控实验先行（实例形准入单独翻转 CP.inst、CH 静态锚不翻）；行号按锚点名重验（FieldCopies 于 build.rs，锚=结构名）。

- [ ] 1.1 复核探针（CP 类重编双腿）+ 定位 Chain proof 的 dup 读取处（为何 `dup_x1` 不入读）；门控实验转录存证据。
- [ ] 1.2 冻结锚与负例双腿：CP（探针源）+ 混合链（静态+实例同方法）；负例=跨对象链（`o1.a = o2.b = 5`）、非 putfield 消费形。
- [ ] 2.1 实现实例形 Chain（dup_x1 交织 receiver 判据 + 三 aload_0 SSA 同一性）；既有 Chain/Receiver 判据逐字不动。
- [ ] 2.2 对照测试：CP 恢复（重编+`-Xverify:all` 行为一致）；`recover_chained_field_assignment` 套件零回退；负例拒绝逐字。
- [ ] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。
- [ ] 3.2 root 独立复核：门控、判据最小性、锚/负例实测、账本（探针边界关闭）。（留 root）
