# 全量工作区 v2：语料清单遗漏的失败观察

root 实际 run-v2 / session61262 终止 exit1；67命令/134 raw，首seed第65条命令的 corpus_files_match_the_recorded_fingerprint 失败。4个有意新增的夹具未登记；未改写旧语料。未进入第二seed，不能称完整工作区通过。finally exact cargo clean0/target absent；4860来源前后恒同。

归档 capture.tar.gz 为1283235字节，SHA256 `262c12762a6061a7ad47a144121dc8d2094bd6f15b7babce787eeca601030719`，147成员全部复开核字节及摘要，其中134 raw。包含实际receipt/metadata、旧清单/4夹具、v4工具与原始source pins。

候选清单只从2193增至2197条；旧清单在归档内，候选清单为仓库tests/fixtures/corpus-fingerprint.json（SHA256 be677275ac72c0cb57d9702314d73cdd40f343459f468f28bda666be5d55c09a）；patch/pins/report和root独立核对同存。4个BLAKE3取自实际Rust失败输出，尚待fresh P5复算；其他记录及分类恒同。private pins的base_revision指历史产品内容来源，真实工作区head为845cfdc8d。

归档私有v1工具第一次实际失败是硬编码v4 pins.json摘要抄录错误；root只修正该工具pin并成功重跑，未修改产品代码，见root-review.json。该失败归档不代替后续fresh局部门、双seed完整工作区、产品自身CI或最终主线审计。

登记时的“待fresh P5”状态已由[实际focused-v11](../final-focused-root-v11/README.md)的5 passed/1 ignored及2197文件复算完成；该组件成功不改变本归档的workspace-v2失败分类。
