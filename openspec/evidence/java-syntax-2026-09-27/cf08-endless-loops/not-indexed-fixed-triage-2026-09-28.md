# CF-08 固定 `TestNotIndexedLoop`：二层切片后的物理阻碍

主线 `d64f2c68` 后，固定 `NotIndexedLoop.java`（SHA-256 `508c2640dcf1ef594fb491f3cbc30d8d2a9b758aaab8dd928fa8e36aa080965d`）按 `javac --release 8 -g:none` 重编的 class 与归档一致（SHA-256 `b1e6bd92f8fe92e726e7a2a95461327dd3f7abe65ee7f451f1225777ed7ea6d4`）。原类和固定 JADX 完整源码重编、`java -Xverify:all` 运行均输出 `null / null / f / h`。当前 Jarde 的完整源码仍因 `local 2 crosses a quoted fallback region` 缺返回而不能编译；Region 首次拒绝在内层 BCI 4 `jre_region_arms_do_not_meet`，未覆盖 `[64,25,38,55,58]`。这与已恢复的 `TwoLevelIf.pick` 是不同的物理值流，不把已过最小样例当成固定测试通过。

固定方法的正常 CFG（块首 BCI）是 `0→{4,67}; 4→{11,16}; 11→64; 16→19; 19→{25,38}; 25→64; 38→{55,58}; 55→64; 58→19; 64→69; 67→69; 69→{73,77}; 73→77`。`JRE_JOIN_PROBE` 给外层/内层立即后支配点 69/64。自然循环为 `{19,38,58}`，两个出口为 25/55，空兄弟臂为 11，三路在 64 汇合；64 的唯一正常后继是 69。现有 `continue_inner_join_arm` 对单块 `goto 69` 的尾续接和三个准确前驱已有证书，前提是循环先结构化。

现有 `effectful_dual_exit_loop` 先要求出口块恰为 `[Push, Invoke, Store, Transfer]` 且调用直接产生写入值。固定出口 BCI 25 是六条 `[new, dup, ldc, invokespecial, astore, goto]`：构造调用返回 void，保存的是经 `dup` 留下并完成初始化的对象，而不是调用的返回值。之后该证书还明文拒绝循环体调用；固定 BCI 38 的体在数组元素写入 local 2 后执行 `File.getName()`（44）和 `String.equals()`（49），由 BCI 52 分支选择 55 break 或 58 latch。单独删除任一拒绝门都不足以恢复固定类。

内层 local 2 在 BCI 64 的三份物理写入候选为 12（null）、34（构造对象）、42（数组元素）；64 只含跳转，外层 BCI 67 另写 null，真正的判空和返回读取在 69/77。因此现有“内层 φ 在本 join 立即被一次 Load 消费”的门槛也不适用。下一实现必须用实际 SSA 核实并证明单次 `φ64→φ69` 转送、外层 null 入值与后续读取，而非放松局部绑定或仅凭 slot 相同宣称等价。这三个点应作为同一固定类恢复里程碑验证。
