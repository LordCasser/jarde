# CF-16 有分支正文的 finally：当前主线重放

root 用主线 `de381b72` 的 CLI（SHA-256 `e2c9e66b56ca335e6460cbfece233207eba8381d91c4eed472856893fbe34ab6`）独立恢复冻结 `ImplicitCleanup.class`（SHA-256 `924437916dc278eefe3b83cdcf3bad14bfb3cf8b9e44c027786a89f0712247b6`）。Jarde 完整源码 SHA-256 为 `f109a468f0d6892e20a30a4f4774039f5a82812cfe8e78ac0777e6351494e586`；`run()` 包含受保护的 `if`/`throw`、唯一正常返回和一份 `finally { cleanup(); }`，无 fallback。`recover --evidence source_map` 的来源集合恰好覆盖该方法全部 17 个真实指令 BCI：`0,3,6,7,10,11,14,15,16,19,20,23,24,25,26,29,30`。

原、固定 JADX 与当前 Jarde 三份完整类均以 `javac --release 8 -g:none` 重编，并以同一个四路径 runner 在 `java -Xverify:all` 下运行。JADX 输出保留其 `defpackage`，只为 runner 加同包声明；未改反编译类文本。原/Jarde 输出逐行相同：正常返回 `2:29`，try 异常 `IllegalArgumentException/原对象/19`，cleanup 覆盖正常返回 `IllegalStateException/原对象/29`，cleanup 覆盖 try 异常 `IllegalStateException/原对象/19`。固定 JADX 在第三条把 trace 写成 `299`，重复执行清理，不是语义 oracle。

同一 CLI 对 verifier 有效的扩围异常表反例（class SHA-256 `c5407d3f29b135818f003e24b218b5e4b7348d261665f0c71605471a7492935e`）仍输出 `@bytecode`，没有错误折叠为单次 `finally`；原 CF-16 `FinallyOnce.handled` 仍在 BCI `0 8 18 31 65` 引用整方法。此重放确认结构化正文首片已在主线可用，同时保留 CF-16 的 typed catch/共享 catch-all 与局部范围差距；不勾销 [recover-proved-finally-cleanup](../../../changes/recover-proved-finally-cleanup/tasks.md) 余项或宣称整单元追平。
