# Child-array 控制方案勘误与 verifier-only 启动器

## 勘误范围

本文件补充 [child-control-construction-plan-v1.md](/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-covariant-child-array-initializers/results/child-control-construction-plan-v1.md)，不覆盖、不改写 v1，也不改变其 SHA。v1 中 child-parent 间副作用序列 `iconst_1; invokestatic #11; pop` 长度误写成 4 字节，正确长度为 **5 字节**：`iconst_1` 1 字节、`invokestatic` 3 字节（opcode + 2 字节 CP index）、`pop` 1 字节。

## 对其余字节与偏移的复核

以两条 direct `Main.class` 的真实 `ownGridDirect()` Code 属性为准，javac8 与 javac23 的 `max_stack=9`、`max_locals=0`、`code_length=47`、异常表 0 项、Code 子属性 0 项、Code attribute length 59。两腿都满足下表的最大栈深；所有插入只改测试变体的 Code 字节和 `code_length`/Code attribute length，不写产品或 fixture。

| 变体 | Code 字节变化 | 新 Code 长度 | 新 Code attribute length | 插入点后的原 BCI 映射 | 最大栈深 |
|---|---:|---:|---:|---|---:|
| 父 index 交换 | 两个 `iconst` opcode 原位替换；净 0 | 47 | 59 | 不变 | 9 |
| child reader `dup; pop` | +2 | 49 | 61 | BCI 24 起 +2；原 parent store 24→26、45→47，return 46→48 | 插入点深度 3→4→3，≤9 |
| 区间副作用 `iconst_1; invokestatic; pop` | **+5** | **52** | **64** | BCI 24 起 +5；原 parent store 24→29、45→50，return 46→51 | 插入点深度 3→4→4→3，≤9 |
| Object[] CP 替换 | `anewarray` 的 2 字节 CP operand 替换；净 0 | 47 | 59 | 不变 | 9 |
| invalid verifier control `areturn`→`ireturn` | 原位替换 1 字节 opcode；净 0 | 47 | 59 | 不变 | 9 |

插入点是 child element store BCI 23 之后。上述方法没有分支、异常表或 Code 子属性，因此插入后没有需要同步改写的 branch offset、exception range、StackMapTable 或调试表位置；class reader 会按新的 attribute length 重新计算后续 class-file span。这里的 offset 说的是该变体内部的新 BCI，不应将 parent store 继续报告为原 BCI 24/45。CP 方法调用引用 #11 与 Object class #13 只核实了 javac8 class；javac23 必须继续从自身 constant pool 找真实对应项，尽管其 Code 头部数据相同。

两条 +N 变体的 attribute-length 计算方式是对已有 Code attribute content length 加实际字节增量：`dup;pop` 加 2，effect 三指令序列加 5。Code `code_length` 同样加这些字节数；`max_stack` 无须增加，因为原值 9，而插入点的峰值仅为 4。其余原位改写不改变任何长度或方法后的 class-file offset。

## 最小 VerifyOnly 启动器草案

以下为测试侧启动器设计，不是已编译或运行的代码。把变体 `Main.class` 路径作为参数传入；variant 目录不必放进 parent classpath。专用 loader 总是从该路径定义默认包 `Main`，其它类型（例如未修改的 `Base`、`Mid`）交给 parent loader。启动器显式以 `initialize=false` 加载、调用 `resolveClass`，再查询 `getDeclaredMethods()` 以要求解析方法签名；不调用 `Main.main` 或变体方法，也不触发 Main 的类初始化。

```java
import java.io.IOException;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;

public final class VerifyOnly {
    private static final class VariantLoader extends ClassLoader {
        private final Path mainClass;

        VariantLoader(Path mainClass, ClassLoader parent) {
            super(parent);
            this.mainClass = mainClass;
        }

        @Override
        protected Class<?> loadClass(String name, boolean resolve)
                throws ClassNotFoundException {
            if (!name.equals("Main")) {
                return super.loadClass(name, resolve);
            }
            synchronized (getClassLoadingLock(name)) {
                Class<?> loaded = findLoadedClass(name);
                if (loaded == null) {
                    loaded = findClass(name);
                }
                if (resolve) {
                    resolveClass(loaded);
                }
                return loaded;
            }
        }

        @Override
        protected Class<?> findClass(String name) throws ClassNotFoundException {
            if (!name.equals("Main")) {
                return super.findClass(name);
            }
            try {
                byte[] bytes = Files.readAllBytes(mainClass);
                return defineClass(name, bytes, 0, bytes.length);
            } catch (IOException error) {
                throw new ClassNotFoundException(name, error);
            }
        }

        Class<?> loadResolveAndInspect() throws ClassNotFoundException {
            Class<?> main = Class.forName("Main", false, this);
            resolveClass(main);
            Method[] methods = main.getDeclaredMethods();
            if (methods.length == 0) {
                throw new AssertionError("expected Main methods");
            }
            return main;
        }
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            throw new IllegalArgumentException("<Main.class> <valid|invalid>");
        }
        boolean invalidExpected = args[1].equals("invalid");
        if (!invalidExpected && !args[1].equals("valid")) {
            throw new IllegalArgumentException("mode must be valid or invalid");
        }
        VariantLoader loader = new VariantLoader(
                Path.of(args[0]), VerifyOnly.class.getClassLoader());
        try {
            loader.loadResolveAndInspect();
            if (invalidExpected) {
                throw new AssertionError("invalid Code unexpectedly verified");
            }
        } catch (VerifyError error) {
            if (!invalidExpected) {
                throw error;
            }
        }
    }
}
```

测试侧只把启动器本身和未修改的 companion classes 放在运行 classpath 上，目标变体文件通过参数提供；以 `java -Xverify:all -cp <launcher-and-companions> VerifyOnly <variant-Main.class> valid` 加载有效变体。该命令会执行 VerifyOnly，而不执行 Main 的语义。

## 负对照：确认实际触发 Code verifier

仅“有效类的启动器退出 0”不足以证明指定 Code 被 verifier 检查。应从原始 `ownGridDirect()` 复制另一份仅用于 launcher 自检的 class，原位将 BCI 46 `areturn`（`0xb0`）改为 `ireturn`（`0xac`），保留 `()[[LBase;` 方法 descriptor、栈和其它字节不动。该方法结束时栈顶是 reference，descriptor 也要求 reference；`ireturn` 要求 int，因此这是 verifier 应拒绝的 Code，长度与 Code attribute length 均不变。以同一启动器、同一 `-Xverify:all` 命令和 `invalid` 模式加载该控制，预期在 `defineClass`、`resolveClass` 或方法反射解析期间得到 `VerifyError`。

如果这个 `areturn`→`ireturn` 负对照没有抛 `VerifyError`，说明当前 launcher/加载路径没有证明它验证了该方法 Code；此时不能把 Object[] 变体的“成功 load”记成 JVM verifier 通过，需先修正验证机制或使用确定执行 Code verification、但不调用变义程序的方法。此负对照只用于验证器路径校准，不执行 `ownGridDirect()`；不要把它混入产品恢复断言。

本文只做字节长度复核和启动器设计，没有编译启动器、构造 classfile 变体、运行 `-Xverify:all`、Cargo、Git 或 Java。
