# Child-array VerifyOnly 启动器勘误 v2

## 修正范围

本文只补充 `child-control-plan-correction-v1.md` 的 Java 启动器设计，不改前两份报告或其 hash。上一草案使用 `Path.of`，这是 JDK 11 才加入的 API，不能用于 JDK 8 验收启动器。这里改用 Java 8 已有的 `Paths.get`。同时将变体目录中的 `Main.class` 和所有未改 companions 交给同一个 parent-null `URLClassLoader` 定义，以免默认包类由不同 loader 定义后形成不同 runtime package，进而引入与目标无关的访问差异。

## Java 8 VerifyOnly 最小源码

启动器自身独立放在 launcher classpath；`variantDir` 是临时验证目录，包含变体 `Main.class` 和原样 `Base.class`、`Mid.class`、`DerivedA.class`、`DerivedB.class`、`LocalInterface.class`。这个单一 URLClassLoader 以 bootstrap loader 为 parent：Java 核心类由 bootstrap 解析，Main 与 companions 都由它自己加载。它不调用 `Main.main` 或任何 Main 方法，也不初始化 Main。

```java
import java.io.File;
import java.io.IOException;
import java.lang.reflect.Method;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Paths;

public final class VerifyOnly {
    private static final class VariantLoader extends URLClassLoader {
        VariantLoader(File variantDir) throws IOException {
            super(new URL[] { variantDir.toURI().toURL() }, null);
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
            throw new IllegalArgumentException("<variant-dir> <valid|invalid>");
        }
        boolean invalidExpected;
        if ("invalid".equals(args[1])) {
            invalidExpected = true;
        } else if ("valid".equals(args[1])) {
            invalidExpected = false;
        } else {
            throw new IllegalArgumentException("mode must be valid or invalid");
        }

        try (VariantLoader loader = new VariantLoader(Paths.get(args[0]).toFile())) {
            try {
                loader.loadResolveAndInspect();
                if (invalidExpected) {
                    throw new AssertionError("invalid Code unexpectedly verified");
                }
            } catch (VerifyError error) {
                System.err.println("VERIFY_ERROR " + error.getClass().getName()
                        + ": " + error.getMessage());
                error.printStackTrace(System.err);
                if (!invalidExpected) {
                    throw error;
                }
            }
        }
    }
}
```

用 JDK 8 编译该启动器后，运行时仅将 launcher class 和启动器 classpath 传入；变体目录通过参数传入，避免 parent loader 先加载同名 `Main`。例如：

```text
java -Xverify:all -cp <launcher-classes> VerifyOnly <variant-dir> valid
java -Xverify:all -cp <launcher-classes> VerifyOnly <areturn-to-ireturn-dir> invalid
```

第二条命令的无效目录包含原始 companions，以及仅把 `ownGridDirect()` BCI 46 `areturn` 换成 `ireturn` 的 `Main.class`。descriptor 仍是 `()[[LBase;`，返回栈值仍是 reference；`ireturn` 要求 int，预期 Code verifier 抛出 `VerifyError`。启动器捕获时会把异常类名、消息和原始堆栈写到 stderr。验证记录必须保留这段输出；如果有效类通过而 invalid control 没有产生 `VerifyError`，则验证路径尚未证明它检查了 Code，不能记有效变体已完成 `-Xverify:all` 验证。捕获并打印错误只校准 verifier 路径，不会运行或初始化变义后的 Main。

这是未执行的源码建议；没有编译启动器、运行 Java、构造任何 variant class、运行 Cargo/Git 或修改产品、fixture、spec。
