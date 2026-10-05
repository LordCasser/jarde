# 遗留集合 + finalize() 巡查（2026-10-05 root）

## 发现：JDK ctor + lambda 实参——分配被吞（第 6 族簇第 4 形态，SAFE）

**pqLambda**（`new PriorityQueue<>((a,b) -> b-a)`——lambda 比较器入 JDK ctor，TreeMap/PQ 高频形）：**ctor 分配语句整条被吞**（诊断 "allocation/copy at BCI 0/3 belongs to no shape this run verified"——与匿名类形的 "no safe reference conversion" **诊断异径**），local1 悬空——幸存文本 javac **exit 1（"可能尚未初始化变量local1"——确定赋值错误）= 安全拒形**（非 compilable-wrong）。jadx 全解（lambda 块直呈）。

**第 6 族簇四形态集齐**（同一落点：JDK 参数/ctor 位的转换证据缺失）：
1. 匿名类→JDK 方法参数（`Collections.sort(c, new Comparator…)`）——**critical #18**（幸存可编译）
2. 匿名类→JDK ctor（`new Thread(new Runnable…)`）——SAFE（checked catch 编译错误）
3. String→CharSequence（`Pattern.matcher`）——SAFE（未初始化 local）
4. **lambda→JDK ctor（本发现）——SAFE（未初始化 local）**

四形态中仅匿名→方法参数位产生 compilable-wrong；**ctor 位两种形态幸存文本均含未初始化局部=结构性安全**。三形态合并窄片候选升级为**四形态族簇**（机制同源：JDK 参数类型转换证据）。

## 健康面（负结果）

- **stackOps**（旧 Stack push/peek/pop/size）恢复；
- **dequeOps**（ArrayDeque addFirst/addLast/peekFirst/peekLast + ArrayList 聚合）恢复；
- **finalize()**（try/finally + super.finalize() 清理链）恢复；
- 行为 `[3, 2, 1]/[8, 8, 1]/[6, 99, 3]` 一致（三健康方法经渲染验证）。

## 处置

census 第 6 族条目补形态 4；窄片候选记录四形态族簇；不新增 critical 锚。
