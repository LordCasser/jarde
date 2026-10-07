# 3.1 语料移动账本 + oracle 腿

## 语料指纹（`tests/fixtures/corpus-fingerprint.json`）

`cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`
重渲染后 diff = **+135 行（27 个新文件，纯增，无既有条目改动）**：本片 fixture 的 6 个源文件 +
`data.txt` + 双腿 20 个 class 文件（`WideningProbe` 也在内）。

## fixture 普查（`crates/jarde-reader/src/classfile.rs` 的 pinned 计数）

| 维度 | 基线（父提交） | 本片 | 增量 |
| --- | --- | --- | --- |
| fixtures | 907 | 927 | +20（10 类 × 双腿：`IO`、`IOMidRead`、`IONegatives`、`NestedDepth`+4 伴生、`Returner`、`WideningProbe`） |
| bodies | 3911 | 3957 | +46（每腿 23：`IO` 4、`IOMidRead` 2、`IONegatives` 3、`NestedDepth` 4、`Returner` 3、四个伴生各 1、`WideningProbe` 3） |
| handler records | 344 | 368 | +24（每腿 12：`IO` 4、`IOMidRead` 2、`IONegatives` 6） |
| branch targets | 2453 | 2475 | +22（每腿 11：`IO` 5、`IOMidRead` 2、`IONegatives` 4） |
| subroutines | 8 | 8 | 0 |

计数按该测试自身的"re-measure these counts"约定更新，并附本片来源段落（未删除任何既有段落）。

## 计费表（`tests/p5_bulk_corpus.rs` 的两行 pin）

`record_the_billing_table`（`--ignored --nocapture`）重读后只有 `analysis_steps` 移动：

| 行 | 基线 | 本片 | 归因 |
| --- | --- | --- | --- |
| `many-method-class` | 10539 | **10542** | +3：行集判据的 cheap half 在它读行集的块上**询问** finally 家族（该 case 的 `Guarded` 持有被询问的两形），询问本身计费 |
| `DIRECT_ARM` | 17324 | **17327** | 同上（同一份工作） |
| `SHARED_ARM` | 17324 | **17327** | 同上（同一份工作） |

其余维度（`archive_entries`、`entry_bytes`、`class_bytes`、`class_headers`、`method_bodies`、
`ir_items`、`result_items`、`output_bytes`）**全部未动**，且该测试的文本对照仍然通过——即语料的
**呈现**没有移动，只有那一次询问的计数。三处 pin 的更新都带一行归因注释。

## oracle 腿（强制）

`cargo test --test p3_execution_comparison --all-features --locked -- --ignored` →

```
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.50s
```

即：每个合法 flag 集读同一语料、bulk 入口的正文与行为、P3 findings 的"重编译+执行"回放三腿全
通过，**无过期 oracle 期望需要更新**（本片没有移动任何被该腿比较的正文）。

## 本片自己的双 JDK 回放腿（`tests/recover_io_resource_finally.rs -- --ignored`）

```
test the_anchors_own_class_answers_its_baseline_and_its_boundary_is_stated ... ok
test the_mid_read_guard_answers_what_its_class_answers ... ok
```

内容：锚自身 class 在两腿上 `-Xverify:all` 答 `2/hello|world|`；中读腿（`IOMidRead` 呈现文本剥离
注释后，用 `javac --release 8` 与真 javac 8 各自编译）在 `-Xverify:all` 下由同一驱动跑出
`normal=3` 与 `caught=read 3 failed closed=true`，与原 class 逐字相同；同时逐字钉住
`IO` 整类文本在 `readAll` 边界存在时**不可编译**（边界如实陈述，后续片落地时该断言按新终态显式
翻转）。
