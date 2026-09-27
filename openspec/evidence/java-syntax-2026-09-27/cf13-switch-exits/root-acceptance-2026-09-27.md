# CF-13 主线独立验收

root 在 CF-12/CF-13 合并后的主线构建 CLI `/tmp/jarde-root-cli-cf12-cf13`（SHA-256 `5cfa715839a79c53aa4b1ecce244f808a911a8c22c1ea91c6f948ad42c219dae`），从归档原 class 独立输出完整 Jarde 源码，SHA-256 `31d8374396c39b604bdc09923c6fa556b9823f72d3a1e0b68c15b3f48b363b2c` 与实现报告一致。源码无 `@bytecode`，case 1 的 `continue` 后无不可达 `break`；共享 `score + 3` 与循环更新各出现一次。

root 重新用 `javac --release 8 -g:none` 编译原输入与 Jarde 完整源码，两个产物以 `java -Xverify:all` 执行同一 runner 均输出 `38`。固定 JADX 的完整源码仍在第 16 行因 `continue;` 后的 `break;` 编译失败；不把它当行为 oracle。root 对同一 CLI 的单方法 source-map JSON 核对：BCI 9 是 selector `i`，BCI 12 对应 `switch (i % 3)`，BCI 49 对应 `continue;`，BCI 55 对应唯一共享语句，BCI 58 对应唯一循环更新。完成的 Region 树仍经生产路径的 `overlapping_owner` 检查；定向正反例与预算/取消回归另由测试核验。
