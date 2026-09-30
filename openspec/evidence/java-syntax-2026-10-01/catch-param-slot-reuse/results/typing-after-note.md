# 恢复输出（变更后）

`S1-typing.after.java`：主线修复（caught 值归属呈现）后，固定 fixture `S1.class` 的
`class-source` 完整输出。与 [`S1.jarde.java`](S1.jarde.java)（巡查基线）逐字对比的唯一差异：

```diff
         } catch (java.lang.IllegalStateException local0) {
-            // @bytecode 43 40 39
-            // the parameter 0 of the invocation at BCI 40 is declared `java.lang.IllegalStateException` presents `S1` but the invocation requires `java.lang.IllegalStateException` and this layer has no safe reference conversion evidence
+            return tag(local0);
         }
```

`twrHelper` 从"子句头正确、handler 体被引"变为完整恢复；`plainHelper` 与其余成员逐字不变。

重编验证：`javac --release 8` 通过；`java -Xverify:all` 正常路径 `done / done`，与原 class 一致
（注入异常路径与三方对照见 [`typing-threeway.md`](typing-threeway.md)）。
