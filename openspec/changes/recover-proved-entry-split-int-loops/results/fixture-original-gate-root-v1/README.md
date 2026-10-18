# 永久 fixture 原始验证 gate

root实际执行17命令并独立复核全部3396具名pin路径，两种物理parser结果一致。五完整类通过JDK8/23 `-Xverify:all`，四个正向方法各18case，共双VM144观察；root用独立闭式32bit公式逐条核对。十个负方法没有执行，四个负类在每VM完成parentless加载、精确CodeSource/成员库存和无参构造；两错误根均按CodeSource拒绝。五份javap与手工BCI/frames/handler账一致。

实际收据SHA `f6ebb66119875977e181d7e1d0aac126114024c9cae46d804bc310d47bc3bd2c`；独立reviewer SHA `1d974d2b2022bf901eb0f964f68d3d2da0cc1bf899e122388e4bf0d0d272480d`。原始gate保持未裁决状态，独立root-review另存，不改写历史。源、工具及CLI前后固定、target absent，运行cwd为空目录。源码/五class已逐字装入 `tests/fixtures/cf08-entry-split`，生成器/命令/完整Code与来源均可重建，尚不代表生产恢复通过。

旧CLI96e仍对四正例和九不可约控制整方法拒绝，legacy保持原既有分块SubroutineEntry/UncoveredBlocks，quality=fallback、representation=mixed。不能为当前测试把旧legacy改为整方法quote。

[capture.tar.gz](capture.tar.gz) 包含全部raw、class、driver、writer、runner、独立审计和v1/v2/v3未执行草稿及静态修正；root逐字与mode重开全部成员，摘要/数量见 [archive-root-review.json](archive-root-review.json)，逐项见 [inventory.json](inventory.json)。草稿跳转错误都在首次执行前修好；本gate没有实际VerifyError，不伪造失败。

本片仅完成OpenSpec1.2输入准备与同次IR事实。生成Java/JADX/Jarde双编译器/双VM验收仍为后续任务；CF08/71/612保持partial。
