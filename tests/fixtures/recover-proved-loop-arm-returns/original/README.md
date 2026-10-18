# 原始 TestNestedLoops5

Java源码逐字来自本地JADX测试（Apache-2.0），完整外类及静态子类均保留。debug/no-debug由javac23 --release8生成，major52；固定源码与完整Code见manifest及OpenSpec基线归档。独立driver提供core/check模式，实际check依赖JadxAssertions major55，Java8加载边界不等于语义失败。不可达早返回仍须恢复，不删源码方法。
