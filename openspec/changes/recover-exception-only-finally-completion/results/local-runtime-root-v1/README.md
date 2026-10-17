## 入口参数 phi 的完整源码及运行验收

最终CLI7287e856…，冻结367B输入，default/all整类原样以javac8/23重编，8次双JVM运行24case与4次原始运行12case逐字相同。null路径得NPE，两个非null异常保持对象identity，三路径trace均78。全部三方法物理身份/index/BCI及UTF-8映射、default/all正文/map/质量相同；34命令/68raw，资源守卫无停止，来源恒同，target不存在。

JADX1.5.6同输入2次编译成功、4次运行均首case失败：trace=8，期望78；没有声称跑完后两case。归档含完整原始输出、源码/class、工具pin、collector与独立driver；每成员复开，SHA见root-review.json。
