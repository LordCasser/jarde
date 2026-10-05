# 双括号初始化巡查（2026-10-05 root）

## 健康面

无捕获形（`DB$1`）：`class DB$1 extends ArrayList` + 实例块内联 ctor（super 后 `this.add("a"); this.add("b");`）——**合法可编译**；宿主 `dbl = new DB$1()` / `withCapture → new DB$2(arg0)` 全恢复。

## 发现：捕获形伴生的 super 前赋值（第 12 个新证缺口，呈现缺陷级）

`DB$2`（捕获 `s`）渲染为：

```java
DB$2(java.lang.String arg1) {
    this.val$s = arg1;   // ← super 之前——标准 Java 非法！
    super();
    this.add(this.val$s);
}
```

拼接 `javac` exit 1——"灵活构造器是预览功能"（Java 21+ preview；8/标准版禁止 super 前引用 this）。**机制**：javac 8 对捕获形伴生发射 `aload_0; aload_1; putfield val$s; aload_0; invokespecial super`——val$ 赋值在字节码层先于 super（合法），但源级不可表达（除非灵活构造器）。**jadx 有解**：识别整个模式（匿名子类+纯实例块体+单分配点）在分配点直接呈现源级双括号形 `new ArrayList<String>() {{ add(str); }}`（[results/jadx-DB.java](results/jadx-DB.java)——含捕获形，完全绕开 ctor 排序）。

## 处置

第 12 个新证窄缺口（呈现域二选一：A. 伴生 ctor 重排——当 pre-super 语句仅为 val$x 赋值且其值不流入 super() 实参时可证安全（super(); this.val$x = argN;）；B. jadx 式分配点双括号形）。登记 + 立窄片。
