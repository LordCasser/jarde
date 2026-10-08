# 分支体锁卫的呈现（recover-branching-guard-body）

## Why

[nested-lock 片登记边界 #3](../recover-nested-lock-finally-bodies/)（测量钉死）：guard 体内含**分支**时（`nestedLocksBranching`，BCI 55 拒）——释放副本落入自己的块，canonical 图把方法尾 `return` 融合进该块，**void 完成形的 transfer 无后继块**。同行为无该布局的 `nestedLocksThrowing` 可呈现——配对即测量边界线。这是 guard 族的最后一个已测明形状缺口。

## What Changes

- lock/resource-guard 证书的完成形扩展：void 完成形的 transfer 终块若被融合进释放副本所在块，读其**尾 span**为 continuation（与 `recover-loop-test-copy-store` 的 fused continuation 同读法先例——transfer 块尾 span、直线 Return/Throw 尾约束）；
- 分支体本身由既有 If/区域呈现承接（证书体读取器已呈循环，分支是同级区域形态）；
- MVP：单分支层、void 完成形；多重分支/嵌套 try 保持登记。

## 硬不变量

1. LK/IO/nested-lock/loop-test-copy 五族锚渲染逐字节不变；
2. 直线尾约束保留（尾含 control flow 时仍拒）；
3. 双驱动（normal/异常）解锁序一致。

## 验收

- `nestedLocksBranching` 恢复（0 该族诊断），剥离编译 exit 0、`-Xverify:all` 双驱动与原一致；`nestedLocksThrowing` 零回退；
- 门控实验先行；全门禁 + oracle ignored 腿 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：guard 体内含分支且 void 完成形的锁卫按源码形态呈现，方法行为完整。
