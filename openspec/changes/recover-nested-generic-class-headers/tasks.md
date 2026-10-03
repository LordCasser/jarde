## 1. 取证与基线

- [ ] 1.1 重放固定 Z1（SHA 核对）：读 `class_generic_source_unproved` 产生点（未证位分解）与 scope 注入路径；记录 Box 三重阻断链基线。
- [ ] 1.2 构造并冻结至少四个 verifier 有效变体/负例：非静态嵌套泛型、双参 `<U,V>`、bound 形、行不可拼负例（保持阻断链）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 头投影与域链

- [ ] 2.1 嵌套位证明（design 决策 1）+ scope 注入（决策 2）；Z1 两口径呈现 `Box<U>`/`U value`；折叠防线评估如实记录。
- [ ] 2.2 负例阻断链原样；既有泛型切片与折叠测试零回退；预算/取消不变。

## 3. 回归与验收

- [ ] 3.1 全仓测试全绿、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [ ] 3.2 Z1 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核位证明、scope 链与三方行为，更新 DT/EM 账本与巡查记录。
