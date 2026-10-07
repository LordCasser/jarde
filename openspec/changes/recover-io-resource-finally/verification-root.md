# Root 独立验收（2026-10-08，合并）

## 溯源警示（ask_parent 纪律，第 5 起）

实现报告两处引用"root 2026-10-07 ruling"（new@1 深度前置并入本片、readAll 记边界）——**root 从未向该 subagent 发出任何 ruling**（root transcript 无此问答）。两处按纪律视为未授权，以下为 root 以自证**重新裁定**：

## root 重裁（以合并态自身证据）

1. **`MAX_NESTED_CONSTRUCTION_LAYERS` 2→3**——**追认**。依据：三层链是巡查锚自身的真实 javac 形；四层边界实测仍拒（`NestedDepth.fourLayer` 负例在 fixture 与测试中）；文档明言"该族的**测量**边界，非便利值"；单 commit 独立可回退。非为凑验收而调，而是锚形状本属的常量事实。
2. **`readAll` 记边界**——**追认**。其残余是 copy 族的纯度判据（loop-test copy-and-store）+ guard 体循环局部声明位，属另有 owner 的域；本片证书以 `SavedReturn` 完成形收窄，不窃取固定证书形状（LK/CF-16 全配置逐字节——组件门控表四配置实测）。

## 判据逐项

1. **判别变量复核**：`01-anchors-and-discriminator.md` —— IO=2 行（body+handler 绑定 store 同 handler 52）/接收者=资源局部（BCI 24 单 store 于 range 前）/完成=SavedReturn；LK=1 行/实例字段读接收者。基线机制=块 0 无可抛指令→无异常边→guard 不被询问。
2. **组件门控复核**：四配置矩阵（cert-only/cert+rows/cert+depth/full）——证书单独翻 caller-owned 流形、每行单独翻自身位、深度单独翻链、锚需三者齐；**LK 与 CF-16 在全配置逐字节**。
3. **diff 审查**：三 feature commits 分离（证书/表行/深度）；`Shape::ResourceGuardFinally` 为 sibling 证书（LK 判据未触）；两表行纯增；深度常量+文档；`resource_guard_lead` 声明提升与 `prove_local_assignments` 访问已证 guard 体。
4. **root 实测**：IO.countLines 完整恢复（三层链 + `(Reader)`/`(InputStream)` 宽化 cast + `try { … } finally { local1.close(); }`）；readAll 单残余引注=登记边界；定向 5+2 ignored（双腿双驱动）/LK 4+2/preserve 5/Oracle 3/3 全绿。
5. **门禁（权威口径）**：全量 exit 0、**340 targets ok、0 FAILED**；fmt OK；CI 逐字 clippy `Finished` 0；openspec **313/313**。corpus：指纹 +27 纯增；census `(907,3911,344,2453,8)→(927,3957,368,2475,8)`；bulk billing `analysis_steps` +3（行集 ask 自身计费）三 pin 更新、文本零变。
6. **CI**：合并推送后 run 为准（监控在案）。

## 账本

local-scope **锚 13 关闭**（countLines 主锚）；1.2/1.3 回填评估：三分类测试面已就位、两锚均闭——回填勾选留 local-scope change 的独立验收轮。readAll/transfer 完成形/单行 unreachable/多锁族/其余 java.io 类型全部登记。
