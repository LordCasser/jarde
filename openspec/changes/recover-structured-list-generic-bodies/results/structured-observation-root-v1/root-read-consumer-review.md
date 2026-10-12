# Root真实参数读取与源用途对应

Root用独立acceptor的平衡Debug parser读actual attempt1 census与Program，并核完整SSA：

| 参数 | physical Load | Stack output | 唯一 receiver consumer | AST source |
|---|---:|---|---|---|
| slot1 | 0 | Stack0 V24 | Invoke1 List.iterator() | outerForEach.iterable Local l1 primary0 |
| slot2 | 27 | Stack0 V35 | Invoke28 List.iterator() | innerForEach.iterable Local l2 primary27 |
| slot2 | 76 | Stack0 V51 | Invoke79 List.add(Object)Z | Call receiver Local l2 primary76 |
| slot1 | 88 | Stack0 V54 | Invoke91 List.remove(Object)Z | Call receiver Local l1 primary88 |
| slot2 | 103 | Stack0 V38 | Invoke104 List.size()I | Call receiver Local l2 primary103 |
| slot1 | 112 | Stack0 V47 | Invoke113 List.clear()V | Call receiver Local l1 primary112 |

两个parameter local原始SSA值分别V1/V2，全部6read无write。addarg physicalLoad77是Local4 V34→Stack1 V52，同AST实际String声明s1、赋值真实checkcast22；removeLoad89是Local6 V43→Stack1V55，对应String声明s2初始化checkcast52。两真实cast分别V32→V33、V41→V42；并非调用点Derived包装。

该表仅解释固定实际方法：实现必须对每次物理事实动态推导，不hardcode这些BCI/slot/name，也不能把6条表直接当通用证明。单Stack输出和唯一消费必须验证完整uses而非只看本表中两指令相邻。源foreach吞掉iterator调用的形态不意味着没有物理事实；保留Program iterable的primary来源与原始Invoke/SSA足够建立当前必要有限关联。剩余参数泛型类型和add元素源可赋值性仍待实现及完整反例验收。
