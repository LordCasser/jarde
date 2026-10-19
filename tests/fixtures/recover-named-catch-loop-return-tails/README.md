# TestBreakInLoop5 完整原件

保留上游完整 Java（outer JUnit、nested TestCls 和原 check）、debug/no-debug 的全部 outer/target class 与同一完整 class JAR。没有裁剪方法、重写源码或为了通过恢复而更改 oracle。

来源为本地 JADX integration/loops/TestBreakInLoop5.java，源文件及每个二进制的 bytes/SHA、冻结输入路径见 manifest.json；许可证及 NOTICE 在 licenses/。产品实现只参考算法，没有复制 JADX 实现代码。

真实三方基线入口：openspec/evidence/java-syntax-2026-10-19/cf09-loop4-loop5-original-root-v1/README.md。原件由 JDK23 --release 8 编译为 major52；原生 javac8 的完整源码尝试因现有测试框架 major55/52 边界失败，收据原样保留。check 在 JDK23 实际通过，JDK8 的框架边界与独立 test cases 分开记录。

本目录只是冻结输入，不代表该特性实现或验收已通过。
