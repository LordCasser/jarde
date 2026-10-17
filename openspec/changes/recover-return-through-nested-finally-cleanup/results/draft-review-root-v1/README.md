# 私有初稿的根审查：未接受实现

该补丁只作审查证据，未应用到产品、未编译、未算完成。root 对照原图与全部代码，发现需要先修正：

1. 原稿遗漏 outer handler 正常完成时的 out 副本 `[46,54)`。只比较正常 out `[13,21)`、inner 异常 out `[25,33)` 和 nested 异常 out `[59,67)`，而把 `[37,57)` 整块当作已解释，可能静默丢弃修改后的 receiver/参数/额外副作用。须从 nested.end 到 outer handler transfer 推导第四份副本，独立闭合并比较完整操作及本副本 SSA。
2. effect 的 expected 若直接由同一输入异常表计算，只证明事实一致，不独立证明 out 不受保护。必须按每份 body/in/out/binding 的语义角色断言准确 handler 集，再核 effects/throw sites；自保护行只罩住不可抛 binding。
3. `Facts::in_block` 返回 SsaInstruction，不能访问 Step 的 `.instruction` 字段。API 类型需据源码修正后实际编译。
4. fused block 的切片必须完整且无重复 owner；非结构化分支的 declare/body/return/cleanup/undeclared/最终 push 均需同一 checkpoint 原子回滚。

Luna 已收到修正要求。原稿与最终产品必须区分；不能把描述符设计、静态审查或本文件当成正向恢复或运行验收。

原稿 SHA256：`9bc4f6aeb5353dd9f0d41bf794a5bc940eb926c08b89aacaef49f15d1fb77037`

原稿按 `unaccepted-candidate.patch.gz` 无损保存，解压后 SHA 与上述一致；保留 diff context 空格，不作为产品 patch 应用。
