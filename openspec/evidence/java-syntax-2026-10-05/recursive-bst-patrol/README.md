# 递归 BST 巡查（2026-10-05 root，负结果——树结构骨干全过）

## 探针

[fixture/BT.java](fixture/BT.java)（`--release 8`）：**递归结果写回字段**（`n.left = insert(n.left, v)`——引用字段递归赋值路径）、递归只读查找（if-else 链）、递归累积（inorder SB 拼接）、静态根字段更新（`root = insert(root, v)`）、for-each 批量建树。

## 结果：**健康，无缺口**

- **insert 完整恢复**：null 基例+new、双臂递归**字段写回**（`arg0.left = insert(arg0.left, arg1)`——引用字段赋递归调用结果不触发任何失败面）、尾 return n；
- contains（if-else 链递归）、inorder（SB 递归拼接）、add（静态字段读写）、BT$Node 伴生（quotes=0）全恢复；
- 两条方法内尾随注记（"live block(s) reachable only through edges the normal-flow view leaves out"，insert BCI 56/57 与 contains 43）——**幸存文本完整、行为一致**=呈现质量数据点（非缺口；与已知死指令对/aload_1;pop 报告质量点同域）；
- 行为 `1,2,3,4,5,6,7,8,9,/true/false` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。递归数据结构域（写回/只读/累积/静态根）确认覆盖。
