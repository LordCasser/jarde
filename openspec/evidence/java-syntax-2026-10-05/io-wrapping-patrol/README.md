# IO 流包装链巡查（2026-10-06 root）——resource-across-finally = local-scope 第 13 锚

## 探针

[fixture/IO.java](fixture/IO.java)（`--release 8`）：三层包装链（`new BufferedReader(new InputStreamReader(new FileInputStream(path), "UTF-8"))`）+ readLine null 循环 + try-finally close；FileReader 手动 char 循环 + `(char)` cast + SB 累积 + close。

## 发现

- **两方法均整方法 crosses 拒（SAFE）**："local 1 crosses a quoted fallback region"——resource 局部（`r`/`fr`）的 def-use（声明→循环内多次调用→finally 的 close()）穿 try-finally 区；空 body + 非 void 缺 return → 编译失败 = SAFE；
- **机制**：resource-across-finally——与第 11 锚（ThreadLocal save/restore）同族变体：跨 finally 的不是保存值而是**资源句柄本身**（循环读 + finally 关闭的双区域使用）；jadx 完整解（try/finally + readLine 循环）；
- 三层包装的 ctor 链（潜在 family-6 宽化位：FileInputStream→InputStream @ InputStreamReader ctor、InputStreamReader→Reader @ BufferedReader ctor）**未独立出现**——整方法 crosses 拒把包装链一并吞掉（先触发的 crosses 更外层）；若 local-scope 片先行落地，这些位点会浮出，届时并入第 6 族簇位表。

## 处置

**local-scope 片第 13 锚**（resource-across-finally：IO 样板 = 真实代码最高频资源管理形）。附带预记：local-scope 落地后 IO 宽化位点入第 6 族簇。
