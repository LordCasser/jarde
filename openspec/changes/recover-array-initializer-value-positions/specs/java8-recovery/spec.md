## ADDED Requirements

### Requirement: 数组初始化 dance 值 SHALL 呈现于任意单一消费方位

当 `new T[]{…}` 的初始化 dance（`newarray` + `dup` + 内层存储序列）是该数组的唯一写者、且其 dup 引用**恰好有一个**后续消费方（包括但不限于：预存数组元素存储的值位、新鲜数组的立即下标/长度接收者）时，系统 SHALL 在该消费位呈现 `new T[]{…}` 显式形——与既有的四个合格位（局部赋值、字段赋值、方法实参、外层初始化器元素）同形。

dance 值存在**多个**消费方时 SHALL 保持既有拒绝；既有四位合格位、裸数组立即消费、锯齿初始化器族的呈现 SHALL 逐字不变。

#### Scenario: 元素存储 RHS 恢复

- **WHEN** `partial[0] = new int[]{7};`（预存数组的 aastore 值位；dance 单写者、dup 单读者）经 `class-source` 呈现
- **THEN** 语句恢复呈现（0 引注）；整类渲染源集 `javac --release 8` exit 0、`main` 输出与原 class 逐行一致

#### Scenario: 立即下标恢复

- **WHEN** `return new int[]{9}[0];`（新鲜 dance 数组的立即 arrayload）经呈现
- **THEN** 恢复为 `return new int[]{9}[0];` 显式形、行为一致

#### Scenario: 多消费方仍拒绝

- **WHEN** 同一 dance 值被两处消费（负例探针）
- **THEN** 保持既有拒绝——判据是单读者语义，不是无条件接受

#### Scenario: 既有位零回退

- **WHEN** 四位合格位、裸数组立即消费（`new int[2].length`）、锯齿初始化器（字段与方法返回位）经呈现
- **THEN** 既有呈现逐字不变
