# 空控制结构巡查（2026-10-05 root，负结果——副作用判别精确）

## 探针

[fixture/EC.java](fixture/EC.java)（`--release 8`）：空 then、双空 if/else、**副作用条件空循环体**（`while(bump()){}`）、空 for 体（i++ 副作用）、`try{}finally{return}`。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）——副作用判别精确

- **纯条件空结构被消除**（`if(x>0){}`/双空 → 条件纯无副作用，消除等价）；
- **副作用条件空循环保留**（`while (bump()) { }`——bump() 有副作用必须执行，呈现保留空体循环——精确）；
- **空 for 体保留**（`local0++` 副作用）；`try{} finally { return 5; }` → `return 5;`（空 try 下 finally 体恒执行——等价归一）；
- 行为 `1/2/3/9/5` 逐行 IDENTICAL（emptyLoop 的 calls 副作用链 3 次执行精确）。

## 处置

负结果归档，不立 spec。空结构族（纯消除/副作用保留的判别）确认覆盖。
