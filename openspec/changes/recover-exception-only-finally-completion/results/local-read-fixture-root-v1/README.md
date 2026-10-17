# Builder local-read rollback 输入

另从完整ExceptionOnlyLocal.java实际javac23 --release8 -g:none生成367B独立类，保留默认constructor/mark/run；双JVM -Xverify:all加载反射通过，未执行目标。四命令八raw、实际javap、完整源/class均保留。新增1class/3Code/1handler/0branch/jsr已由独立Code解析核对，root复制到独立新文件，未改变已冻结的正例或红基线。

这是为了在真实参数读取AST上验证Builder拒绝时回滚，测试内注入decl/undeclared只是表示失败边界条件，不冒称原Java非法。尚未以该收据宣称新Rust测试通过；实际结果另记final-focused。
