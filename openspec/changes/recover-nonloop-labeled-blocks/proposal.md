## Why

[非循环标签块巡查](../../evidence/java-syntax-2026-10-05/labeled-block-patrol/README.md)实证：`outer: { inner: { if(x==1) break outer; … } … }`——**break 到非循环标签**（跳过中间 join/桥）拒绝："the intermediate join, bridge and outer conditional do not form a closed value"。**jadx 有解**：if 嵌套等价形（`break outer` → 外层 skip 剩余语句、`break inner` → 内层 skip），stub 替换后**行为逐行一致**（`100/110/111` 验证）。

**判别（同巡查）**：循环标签 `break loopN`（既有通道恢复）与非循环标签是**不同落点**；无限循环三形（`while(true)`/`for(;;)`/`do…while(true)`）全部以等价 do-while 归一化恢复——缺口仅非循环标签。

## What Changes

当 break 的目标是**非循环标签块**且跳转目标（块的结束 join）可证时，把标签块呈现为 **if 嵌套等价形**：每个 `break label` 变为"跳过该标签块剩余部分"的条件结构（与 jadx 同构、与 finally-return 的 if/else 等价退化同族——**既有等价退化哲学**的又一应用，非新机制）。

## Impact

- **代码**：`crates/jarde-java/src/build.rs`/`region.rs` 的 break-标签呈现区（"intermediate join, bridge and outer conditional do not form a closed value" 发出处，task 1.1 定位）。
- **测试**：`LB` fixture（判别已冻结：标签拒/无限循环恢复）+ 循环标签零回退。
- **账本**：summary.md 登记行关闭。

## Non-Goals

- **不**做源级标签还原（呈现 if 嵌套等价形即可——jadx 同构）；
- **不**动循环标签 `break loopN` 通道（零回退锚）；
- **不**处理 continue 到非循环标签（非法源——javac 拒编，无字节码形）。
