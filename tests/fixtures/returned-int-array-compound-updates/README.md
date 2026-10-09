# 返回 int 数组复合更新 fixture

完整11成员原类及固定Runner/oracle按字节复制自root实际双JDK原程序、fresh JADX四腿与当前Jarde失败基线。v8为Corretto8u432，v23为OpenJDK23.0.1；编译输入为source8/target8/g:none、空CP/SP，工具与raw证据见来源manifest。

八份typed controls仅改scalar参数descriptor一字节；boolean索引/RHS必须拒绝，char索引/RHS允许既有Java int提升。两对应真实JVM均-Xverify:all加载/反射成功，未调用这些mutant目标方法，不计运行语义成功。

copy-manifest-v1.json记录完整来源路径和SHA；生成Java必须全类参与重编，运行只用新classes，不借本fixture原class。

六份consumer controls分别在iadd后或store后插入dup;pop，或把scalar末端转换为i2l/lreturn及long descriptor；各对应JVM加载验证通过，目标方法未调用。常驻拒绝/来源测试尚待Rust执行，不称恢复验收。
