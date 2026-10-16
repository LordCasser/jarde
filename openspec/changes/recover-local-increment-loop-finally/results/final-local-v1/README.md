# 最终本地验收

产品02de294f7af2d0798e33d52ce5474e60efe17b03，4929项真实Git源码/模式/字节绑定最终冻结CLI 59902896B/0555，SHA256 fd7b33ce0411854e480186c7ade82ecc21d4401b79d1a0b1bd0d5c7774bbff25。自身CI38872425446仍待终态独审。

本地两份收据共28命令/56raw：前段8命令包含12目标批量link触及1GiB磁盘守卫的-15及成功clean；已过六门保持。续跑20命令按目标逐个完成剩余兼容/CLI/examples/MSRV/两版strict364/0/build/clean。完整完成的唯一测试102 passed/0 failed/8 ignored；中止批次先完成的9个测试被续跑覆盖，单列部分观察，不再累加为新的通过。默认stack、5GiB free/1GiB target磁盘门未放宽。

最终NoCall全源码10命令全0，两份完整1430B text原样经真实javac8、JDK23 --release8编译；未改driver rebuilt-only四次Java8/23输出逐字等于84B oracle。较大原FY/FO/FX另有20命令：两编译器FY/FO各5方法均structured/java、无fallback，整类原样重编和main双JVM共8运行匹配原oracle；FX保留6方法，两份javac仍missing return，未运行重建类，finContinue/finReturn/finBreak保持mixed/fallback。固定FY/FO成功不等于全部finally、JADX或父计划完成。
