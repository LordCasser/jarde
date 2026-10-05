# BitSet 位集运算族巡查（2026-10-06 root，第 150 前沿，混合）

## 探针

[fixture/BT.java](fixture/BT.java)（`--release 8`，新二进制=soundness+monitor 后）：权限掩码（set/clear/nextSetBit 扫描循环）/BitSet 去重计数（cardinality）/位集布尔链（clone+and/or/xor/flip(0,8)）。

## 结果

- **dedup / maskOps 完整恢复**（quotes=0）：for-each set、cardinality、`(BitSet) clone()` cast 链、and/or/xor/flip 作语句、三元返回——往返编译 exit 0、`-Xverify:all` 行为 `3` / `2/4/2/5` 逐行一致；
- **permissions 结构恢复但渲染不可编译（SAFE）**：三个 for 归一 while 全对、nextSetBit 扫描循环+三元返回完整；唯一问题是**槽复用跨类型**——local3 先 `int[]` 后复用为 `StringBuilder`（javac 无 LVT 时同槽异型），声明按首个使用定型导致赋值类型不匹配。归 [preserve-local-scope 槽复用锚家族](../slot-reuse-try-catch-finally-patrol/README.md)（既有域，同因新数据点，不另立）；
- **main = copy 族整方法升级拒**（"the copy at BCI N has no proved local assignment" ×4 级联）——soundness 守卫预期行为（新靶面：new int[]{...} 内联数组实参 + BitSet 链的多消费者舞蹈），SAFE。

## 处置

BitSet 域主体健康；permissions 槽复用异型归 local-scope 片（第 14 数据点）；不立项。
