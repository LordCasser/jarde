# 条件return/落穿 finally：独立完整fixture

目标源码、driver、oracle在生产改动前冻结。原始生成命令与工具pins见OpenSpec change results/planning-baseline-v1。javac23.0.1以--release8 -g:none生成v8；真实Corretto javac8以-source8 -target8 -g:none生成v8-javac8，均456B，SHA分别d061317f86a0bdc1272589b2182e353bf31661a37be7abc9d014ba086048600c与b2c34b111b69943a3b41914dbfb122dfbfb28056fe7e92c2806623117a9f7cbc。

loopless Code50B，唯一any异常行[0,16)→37，三println在21/31/43。driver观察empty/[2]/[3]/null与两种清理抛异常路径，清理恰1次并把value从3000改99，待返回值仍为3000。四次原class Java8/23运行全部匹配134B probe.stdout；当前a89完整生成源码仍缺return，不能用原类成功代替重建。

这是新的独立完整类，不是从原FY删除noCall后的产物，也不声称原FY字节相同。原完整FY/FX全部方法继续另存整体编译结果；本fixture通过不等于父finally计划完成。验收不得改原source/driver/oracle、修生成源码或混入原class/JAR。
