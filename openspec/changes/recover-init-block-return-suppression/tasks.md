## 1. 基线与负例

- [x] 1.1 重放 F2（复用 super-default-patrol fixture，SHA 核对）：定位初始化块语句发射点与语境标识；记录 `<clinit>` 尾 return 基线。
- [x] 1.2 构造并冻结至少三个 verifier 有效变体：多静态块、实例初始化块、静态块分支 return 形；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 语境过滤

- [x] 2.1 初始化块呈现过滤 return（design 决策 1）；F2 整类重编通过、运行一致（`5:42`）；变体恢复；方法/构造器 diff 逐字不变。
- [x] 2.2 既有 `<clinit>` 呈现（enum 折叠等）除 return 外逐字不变。

## 3. 回归与验收

- [x] 3.1 全仓测试全绿、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [x] 3.2 F2 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核语境过滤边界与三方行为，更新账本与巡查记录。
