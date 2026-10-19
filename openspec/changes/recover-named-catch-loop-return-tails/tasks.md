## 1. Frozen evidence

- [x] 1.1 固定完整 Loop5 Java/outer/target/debug/no-debug fixtures 并归档真实 IR；验证 SHA 与已闭合三方原件逐字节一致，row0/[0,123)/handler124/type54、return123 和四 profile 拓扑复核通过。

## 2. Local proof and recovery

- [x] 2.1 Luna 私有实现 prefix+完整Loop+单块return-tail 候选及精确命名行范围证明，root 审查后串行集成；真实 Reader 正例验证 Try body 完整、tail106/return123、normal_exit_bci=None 和全图 owner 唯一。
- [x] 2.2 在 Loop4 分支闭合证明上增加本普通 named row 的边/指令归属、真实latch88与直接guard23的证书join，验证 Loop5 While完整持有外侧46/53/60/67/74/81及append，Loop4 ForHeader及旧goto/终端证明保持不变。
- [x] 2.3 增加错误/额外异常行、越界效果、非唯一末尾 return、foreign entry/exit/owner、重复/遗失、预算、预取消及晚期 Stop 负例；运行测试确认拒绝与完整状态回滚、用量不回退。
- [x] 2.4 复用精确RegionPaths catch绑定及handler自身Stack0 phi契约，修复Builder把catch参数当普通double赋值的冲突；验证完整Loop5 debug/no-debug公开API及错误handler/路径/槽/普通phi拒绝、预算/取消，并保留whole身份及catch逃逸检查。

## 3. Full acceptance

- [ ] 3.1 冻结最终 CLI，重做两原件各8 API/20完整编译尝试和全部成功产物的双 VM 对照、实际 check、wrong-CodeSource 控制、default/all 与物理 source maps；另以已实际冻结的12例Loop5 oracle验证M/g/null/重复点/零/超大数。独立重开 raw，不以单方法、裁剪 outer 或 JDK8 框架不兼容掩盖恢复失败。
- [ ] 3.2 通过完整工作区、旧关键 gates、该产品提交自身 CI；精确核对 named outcomes、source binding 和退出码，所有 Rust 执行保留5 GiB/1 GiB守卫与最终 target 清理收据。
- [ ] 3.3 同步主 spec 并严格验证，更新 handoff/固定71单元账本中的两原件局部状态，提交推送 main；以干净工作区、无 target、无新分支/worktree为最终检查，不扩大 CF09/71/612 完成声明。
