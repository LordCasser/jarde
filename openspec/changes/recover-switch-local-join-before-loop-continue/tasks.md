## 1. 控制流证书

- [x] 1.1 在现有 switch/loop 走访中识别全方法 post-dominator 是当前 loop continue target 的窄形态；证明唯一 switch-local join，并逐 case 分类到 join、准确 continue target 或既有安全 terminal。
- [x] 1.2 用额外入口、多 join、cross-case、嵌套 switch 和普通循环尾部负例确认不授予错误的 continue 证书；预算/取消保持原子。

## 2. Region 与来源

- [x] 2.1 仅对已证 case 路径输出外层 `LoopContinue`，保留 BCI 49 原点；正常 arm 在 local join 停止，BCI 55 的共享语句和 BCI 58 的 loop update 分别由外层续写一次。
- [x] 2.2 验证 completed Region tree 的 block exactly-once、BCI 9/12/49/55/58 的 source map，以及没有 `continue; break;`；现有 `p3_switch_loop_exits` 正反例不退化。

## 3. Java 8 对照验收

- [x] 3.1 用 CF-13 固定原 class 和修后 Jarde 完整源码分别 Java 8 重编、`-Xverify:all`，runner 输出均为 `38`；固定 JADX 的不可达 break 编译失败保留为已知对照事实。
- [x] 3.2 运行相关 switch/loop 定向测试、`cargo fmt --all -- --check`、适用 crate check、`git diff --check`、`openspec validate recover-switch-local-join-before-loop-continue --strict`，记录证据并清理 Cargo target。
