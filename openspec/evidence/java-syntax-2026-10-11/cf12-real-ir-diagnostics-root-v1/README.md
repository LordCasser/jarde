# CF12 真实 IR 诊断

root 临时应用测试观察器，读取已冻结的原上游完整 class，经真实 reader、同次 SSA 与既有 recovery 运行。v2 两个单测实际各 1/0/0；产品文件立即还原，52 个输入指纹与产品 `0ae30a7c8d217522f2e5f5b954aa86c9b59356dd` 恒同。随后在 5 GiB free / target 1 GiB 一秒守卫下执行 cargo clean，target 已移除。独立接受记录见 `acceptance-root-v1.json`，这只是诊断事实，不是产品实现验收或整 CF12 完成。

`diagnostics-v2/0.stderr.raw`：TestSwitch 局部 slot5/index0 仅一写，store29 的真实 stored producer 是 charAt@26 `(I)C`，frame 为 Int，旧决定为 Int。TestSwitchNoDefault slot2/index0 全部五写具有同一 reuse 身份与完整 path：null@0→store1，以及四个直接 String 常量@32/38/44/50→store34/40/46/52；常量 frame 为 Ref(Unknown)，旧决定为 Object。观察器使用同次 reader 的 LVT/LVTT 转换规则，debug 信息仅用于核对，不能独自充当类型证明。

`diagnostics-v2/1.stderr.raw`：条件 fallthrough class 的完整 canonical CFG 有 11 个块、17 条 Normal 边，无不可达块，无 Return/Exception/Call 边。case1 的 32 和 59 均可进入下一 case 的 117，也可经 63、67/92 到 join171；117 自身有来自 switch0 与 32/59 的入边。当前结果真实记录了 117 的 Loop 拒绝、SwitchArmsOverlap 和尾部 UncoveredBlocks。这里没有真实回边；不能把该拒绝当成 Java 源码本来有循环，也不能用只扫描单 successor 的旧探测证明条件 fallthrough。

v1 失败保留：首个 typed 单测 stdout 实际显示通过，但守卫在子进程结束后记录 private 路径时抛出 `relative_to(ROOT)` 错误，第二诊断未运行、commands 为空。v2 只修改 stream 记录为绝对路径，未改变测试和生产算法；成功判据仍独立核实际 exit、测试名、摘要、raw 哈希和 source pins。原失败不计入成功命令。

完整临时前后源码和精确 patch、wrapper 原稿、实际 stdout/stderr、guard 记录与 clean raw 均保留。原始执行记录中的 private 路径是历史运行位置；此目录为保存副本，验证器核副本字节与历史哈希，不回写执行记录。
