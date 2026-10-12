# 外层 try/foreach 的同次 Walker 观察

产品来源 `7e93eb30647338dd8a6ddfc9994392f23457aa3e`。root审查私有v2 patch后实际运行一个既有完整类 integration test：1成功/0失败/0ignored，session64148 terminal0。receipt SHA `2bf1b8ed4b64c7b6609a65869d0db8e7a9c76420d5a15590f7ed19e75cfc3b33`。这是定位证据，不是修复实现或运行语义验收。

`handled(List,List)` 同次图中六个块都存在于 canonical 与 NormalFlowView；自然环header7、blocks[7,16]、latch16、exit47，非irreducible。异常range为[0,47)→50，try join59。protected frame从0开始、boundary59，走到7时prefix[0]，返回body[[0]]及next7。原调用的certified_try_exit仅一次返回false；try最终body[0]、catch[50]，visited[0,50]；末尾59恢复后，[7,16,47]成为最终未owned closure。此前“是否view缺块”的猜测已由本次membership记录排除。

落点是现有try调用方未消费内部continuation。接回循环及range-end纯goto必须继续证明唯一所有权、完整保护范围、handler/外部入口与join、停止/预算；本观察不证明任何放宽Switch gate的方法安全。独立架构评估仍待完成。

10个证据成员打包在capture.tar.gz，manifest逐成员记录bytes/SHA，root重新打开全部成员。包含实际trace patch/runner/pins/receipt及两命令raw；源码快照不重复归档，原来源可由产品Git恢复，candidate由patch及SHA绑定。原region源码逐字恢复，4854 source pins before/after相同，default stack、5GiB/target1GiB/1秒进程组守卫，峰值357199994B，finally exact cargo clean0、target不存在。v1仅私有准备且有类型错误，从未执行；不得混入本次成功证据。
