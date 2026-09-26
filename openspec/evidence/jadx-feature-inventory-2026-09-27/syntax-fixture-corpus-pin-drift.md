# 语法回放 fixture 累计后 corpus pin 漂移

主线 `7536501e` 上独立运行 `cargo test -p jarde-reader --lib --locked`：176 项通过，`repository_class_fixtures_validate_without_false_target_rejections` 只因预期 census `(330, 1730, 160, 1000, 8)` 与实测 `(351, 1804, 160, 1006, 8)` 不同而失败；该测试已逐个成功读取、解码并核对所有 class，失败发生在末尾计数断言。运行 `cargo test --test p5_corpus_fingerprint --locked`：4 项通过，1 项失败，1 项 ignored；失败列出多批已提交但尚未加入 manifest 的语法 fixture，含 `package-info-basic` 和较早的 `static-member-basic`。CF-19 的三臂负例又增加一份 evidence class，但不在 `tests/fixtures` 的 fingerprint 根内。两个 pin 的维护滞后于多项独立语法变更，不能归为 CF-19 单点恢复缺陷。

独立维护任务：待当前 DT-04/DT-11 与 EM-04 冻结 fixture 稳定后，列出全部新增 class 与源码的具体来源及增量，重测 reader census 并更新断言说明；按 `p5_corpus_fingerprint` 的现有再生成命令重建 manifest，逐项审阅新增文件与哈希，然后重跑两个门禁。不得改 reader 解析行为或把失败项目简单忽略；对不应是永久输入的生成残留，应先按各 fixture 的来源规范清理。
