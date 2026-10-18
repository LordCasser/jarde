# 第一轮窄循环实现定向验收

root完整读Luna1051行候选diff、核base/result pins后集成；静态修Option<Type>、重复match arm、零费用checkpoint冗余与测试span usize。真实正向失败又揭示petgraph predecessor枚举不能直接当phi输入顺序，已按Frame logical-input源身份排序；没有改canonical、NormalFlow、jsr身份或公共协议。

保留实际四次运行：v1指纹显式regen1/0通过，取消草稿测试E0308未运行；v2公共5/2失败，修phi顺序和草稿Limits::default.elapsed=0；v3公共7/0全部通过，java旧测试walker因缺新增叶子E0004编译失败；v4只续缺项，java lib507/0（含真实IR proof预算/取消）、Reader全语料census1/0、fingerprint5/0/1旧ignored、fmt与CLI构建成功。不是fresh全workspace，不重跑已通过公共7门。root独立解析全部具名结果、复开v4全部3223源路径及全部raw与guard，保留完整失败；reviewer首次遗漏should-panic后只修parser，未重跑测试。

冻结CLI位于 `/private/tmp/jarde-entry-split-focused-root-v4-20261019/candidate-cli`，63955664B、mode0555、SHA `eb11645e741d944a349080fbd12f18e2e5c70bdc5809f4bdb51a6e3cdec6fb50`。默认栈/5GiB门/target1GiB门/每秒guard通过，四轮峰值676849035B，实际finally cargo clean通过。CLI二进制外置不打入Git，精确收据/源码binding/sha在归档。

[capture.tar.gz](capture.tar.gz) 的全部payload与内嵌库存由root逐字与mode复开；数量/摘要见 [archive-root-review.json](archive-root-review.json)，明细见 [inventory.json](inventory.json)。包含原Luna生产/公库/IRproof准备、root架构审查、四轮format/build收据与失败、全部raw、已接受源码快照和独立物理parser补充。私有草稿/README是当时历史，不替代root实际裁决。

生产核心恢复2.1已接受；2.2真实两处输出与可选map阶段cut、最终完整JADX/Jarde源码对照、fresh native与自身CI仍待完成，不能提前关闭3.1/3.2或整个CF08/71/612。
