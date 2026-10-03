# 消隐前置不变量的 javac 实证（2026-10-04，root）

隐藏 javac 合成成员的前提是源码能让 javac 重新生成同一合成物；擦除桥的可重建性来自类头类型参数投影，不来自平台事实。

## 裸 implements + 隐藏桥 → 硬编译错误
```
public class RawI implements java.lang.Comparable {
    public int compareTo(RawI o) { return 0; }
}
```
```console
$ javac --release 8 -Xlint:-options RawI.java
RawI.java:1: 错误: RawI不是抽象的, 并且未覆盖Comparable中的抽象方法compareTo(Object)
public class RawI implements java.lang.Comparable {
       ^
1 个错误
```

## 参数化 implements + 隐藏桥 → javac 自行重建桥
```
public class ParamI implements java.lang.Comparable<ParamI> {
    public int compareTo(ParamI o) { return 0; }
}
```
```console
$ javac --release 8 -Xlint:-options ParamI.java   # exit=0
$ javap -v -p ParamI.class | grep -c ACC_BRIDGE
1
```
