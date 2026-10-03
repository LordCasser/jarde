# 单静态接口子折叠缺口巡查（2026-10-03）

[lambda 验收](../lambda-inline-patrol/README.md) 中发现的独立缺口（主线 `a7762a92`）。固定转录 [fixture](fixture/)（Y1 家族：单直接静态**接口**子 `Y1$StrFn`〔InnerClasses access_flags 1544=0x0608 static+interface+abstract〕；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），诊断快照 [results/member_family.txt](results/member_family.txt)。

## 表现与判别

| 输入 | 主线 Jarde |
| --- | --- |
| 单直接静态**类**子（M2$Solo，fold 片验收） | 折叠（窄通道回退进入折叠） |
| 多直接静态子（含接口 0x0608，fold 片 FV 变体） | 折叠（`StaticMembers` 通道，接口行已准入） |
| **单直接静态接口子（Y1$StrFn）** | 不折叠——`member_family.projection.state="refused"`，reason="capture proof is incomplete"；`capture.reason="static no-capture target was not proved from the class-level relation"` |

## 根因

fold 片为零回退保留"单静态行走旧窄通道 `Candidate`"的选择；mixed 片把**接口行准入**（0x0608/0x0609）加在 `StaticMembers`（≥2 行）通道上——单接口子落回的窄通道没有该准入，其自身的 no-capture/relation 证书判据不匹配接口形态（接口无构造捕获语义）→ 证明不完备 → 不折叠。

## 处置方向

`recover-single-static-interface-fold`（窄切片）：单静态行的窄通道准入接口形态（0x0608 单可见位——与 `StaticMembers` 行判据同一份），或单静态行统一改道 `StaticMembers`（更简——以 corpus diff 判定等价性）；Y1 家族 jar 输出 `interface StrFn` 折叠呈现、域内 `StrFn` 源码拼写、重编行为一致（`hi!`/`45`/`[b, aa]`/`8`）；M2 类形态与既有全部家族 diff 逐字不变。

原 class 为行为基准。
