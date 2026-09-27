## ADDED Requirements

### Requirement: Direct same-package override annotation projection

对 Java 8 类源码请求，系统 SHALL 仅在准确选定的同一制品直接普通父类及完整成员事实证明子类非合成实例方法确实覆写一个同包可见、非 private/static/final、同名同描述符的父类实例方法时，在子类方法声明前写 `@Override`。该提示 SHALL 作为推导的源码投影，不得冒充 class 文件原有注解或改变物理方法身份；证明不完整时 SHALL 保留原方法声明而不写该提示。

#### Scenario: Same-package direct override

- **WHEN** 同包子类直接继承同一 jar 内选定的普通父类，完整方法表中唯一父方法和子方法同名同描述符，且父方法可覆写
- **THEN** 子类完整类源码在该方法前包含一次 `@Override`，Java 8 重编及运行与原 class 一致

#### Scenario: Private parent name collision

- **WHEN** 直接父类的同名同描述符方法为 private
- **THEN** 子类方法不带 `@Override`，其方法体和物理声明保持原样

#### Scenario: Package-private parent in another package

- **WHEN** 子类位于不同包，而直接父类同名同描述符方法仅为包私有
- **THEN** 子类方法不带 `@Override`，完整源码仍可在 Java 8 下重编

#### Scenario: Incomplete or unsupported relationship

- **WHEN** 父类缺失、定义歧义、成员表不完整、方法为 static/bridge/synthetic、签名关系不在首片证明范围内，或请求预算/取消终止
- **THEN** 系统 SHALL 不凭方法名称写 `@Override`；请求停止时 SHALL 遵循既有停止与输出原子性规则
