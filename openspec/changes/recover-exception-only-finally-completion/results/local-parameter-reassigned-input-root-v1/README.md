# 参数重赋值完整输入

root实际javac23 --release8 -g:none编译完整ExceptionOnlyLocalChanged.java，565B/4Code/1handler/无branch或jsr，双JVM严格加载通过，共4命令8raw，未执行目标。try中参数从replacement字段重赋值，异常入口Local0不再总是同一Entry值，作为非平凡phi拒绝边界；实际Complete IR与proof拒绝由final-focused独立断言，不以JVM合法性代替。
