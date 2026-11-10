# S1 enum arbitrary constructor stop probe

IntBody.java 是本轮受控源夹具，JDK 23.0.1 `javac --release 8 -g -proc:none` 编译为 input.jar。IntBody.class 与归档内主类同字节，用于从同一份物理事实提取构造器候选。两个常量体、int 用户参数进入 arbitrary tail 分支，不经过 zero/String 特例。新目标未执行，不作为独立 holdout；fixture-build 收据见 S1 enum-constructor-stop 检查点。
