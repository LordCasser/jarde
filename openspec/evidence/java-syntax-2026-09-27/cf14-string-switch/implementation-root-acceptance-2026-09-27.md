# CF-14 嵌套 String switch 主线独立验收

root 在 CF-14 实现合入后的主线重建 `jarde-cli`，SHA-256 为 `0c299a1c0143259d5cfe2580ad2ba30e101168ba5c1cec972874952a366086f4`。从三份归档原 class 分别独立生成完整 Jarde 源码：嵌套样本 SHA-256 `269fce1b0f0d365a0954eeb304d95c07e78e1c66b2439bec364af54d1a69af17`，普通样本 `2359246428d71833a0b8fe98ba711e36c826f26779691e6d340f96b6ca6c8832`，独立 hash 用途负例 `dcc3d93b5254e60165b2bbc0623d38ff0a09bcbf4ceec9655057bb70ca5628ba`。三份 Jarde 源码均无 `@bytecode`。

root 重新用 `javac --release 8 -g:none` 编译完整原/JADX/Jarde 类及各自固定 runner，并用 `java -Xverify:all` 对照：嵌套样本五行、普通样本八行三方逐字一致；`ExtraHashUse` 原/Jarde 六行一致，固定 JADX 仍因未定义 `r0` 无法编译。嵌套方法报告为 `quality=structured`、无 fallback，source map 中 BCI 62/71/96/105/111/120/123/125/152/154/156 均可追踪。`p3_patterns` 68/68、格式检查、OpenSpec strict 和 diff check 通过。此验收只关闭已冻结的嵌套续接差距；跨 case/额外入口及独立 hash 消费仍按实施分支的负例边界处理，不宣称 CF-14 所有字符串分派变体已追平。
