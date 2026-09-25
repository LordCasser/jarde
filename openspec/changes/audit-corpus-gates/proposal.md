## Why

`jarde-reader` 的仓库 class 夹具门禁仍固定旧 census（164），现有目录扫描得到 310 个 `.class`；P5 fingerprint 扫描也发现 146 个未登记文件。两项门禁都在表达语料边界，必须先判定输入来源和意图，再决定保留、清理或登记，避免只改数字/重生成索引掩盖夹具漂移。

## What Changes

- 审核全部 146 个未登记文件，按来源、测试消费者、生成方式和是否为临时输出形成可审查分类。
- 先处理证据充分的临时编译残留，再依据最终保留的语料更新 reader census 与 P5 fingerprint 分类/清单。
- 固定失败重放命令和验收顺序；不以机械改计数或盲目运行 fingerprint 再生成器作为验收。

## Capabilities

### New Capabilities

- `corpus-gates`: 维护 class 夹具 census 与 P5 fingerprint 时，要求先审核语料成员及来源，再更新对应门禁。

### Modified Capabilities

无。

## Impact

影响 `crates/jarde-reader` 的 class fixture census、`tests/p5_corpus_fingerprint.rs` 的分类表及 `tests/fixtures/corpus-fingerprint.json`。本 change 只规划后续维护，不包含对生产代码、现有夹具、roadmap 或其他 OpenSpec change 的修改。
