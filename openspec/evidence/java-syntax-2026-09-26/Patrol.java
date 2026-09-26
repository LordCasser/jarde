import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.function.Function;
import java.util.function.Supplier;

public class Patrol {

    // 方法引用四类：static、绑定实例、任意实例、构造器/数组构造器
    static int staticRef(String s) { return s.length(); }

    int boundRef() { return 42; }

    static int anyRef(Patrol p, String s) { return s.hashCode() + p.boundRef(); }

    Supplier<List<String>> listCtor() { return ArrayList::new; }

    Function<Integer, int[]> arrayCtor() { return int[]::new; }

    int dispatch(Supplier<Integer> s, Function<String, Integer> f) { return s.get() + f.apply("x"); }

    int supply() { return 7; }

    int exerciseMethodRefs(Patrol other) {
        return dispatch(this::supply, Patrol::staticRef)
            + dispatch(other::boundRef, this::len)
            + listCtor().get().size()
            + arrayCtor().apply(3).length;
    }

    int len(String s) { return s.length(); }

    // 精确重抛（Java 7 precise rethrow）：catch(Exception) 但 throws 只声明窄类型
    void preciseRethrow(String mode) throws java.text.ParseException, java.io.IOException {
        try {
            if (mode.equals("parse")) throw new java.text.ParseException("p", 0);
            if (mode.equals("io")) throw new IOException("io");
        } catch (Exception e) {
            log(e);
            throw e;
        }
    }

    static void log(Exception e) { }

    // 带标签的普通块（非循环）break
    int labeledBlock(int arg0) {
        int r = 0;
        block: {
            if (arg0 < 0) break block;
            r = arg0 * 2;
            if (arg0 == 0) { r = 1; break block; }
            r += 3;
        }
        return r;
    }

    // 多资源 TWR + 自定义 close 异常（suppressed）
    int multiResource(Path a, Path b) throws IOException {
        try (java.io.BufferedReader r1 = Files.newBufferedReader(a);
             java.io.BufferedReader r2 = Files.newBufferedReader(b)) {
            return r1.read() + r2.read();
        }
    }

    // char 运算与 char switch
    int charOps(char c, int arg1) {
        char up = (char) (c - 32);
        int r = up + arg1;
        switch (up) {
            case 'A': r += 1; break;
            case 'B': r += 2; break;
            default: r += 3;
        }
        return r;
    }

    // 无限循环 + 带标签 continue 跳过内层后语句
    int forever(int arg0) {
        int n = 0;
        outer:
        for (;;) {
            n += 1;
            inner:
            for (int i = 0; i < 3; i++) {
                if (i == 1) continue inner;
                if (i == 2) continue outer;
                if (n > 10) break outer;
                n += i;
            }
            n += 100;
            if (n > 1000) return n;
        }
        return n;
    }

    int unreachableAfter(int arg0) {
        int n = 0;
        outer:
        for (;;) {
            n += 1;
            inner:
            for (int i = 0; i < 3; i++) {
                if (i == 1) continue inner;
                if (i == 2) continue outer;
                if (n > 10) break outer;
                n += i;
            }
            n += 100;
            if (n > 1000) return -n;
        }
        return n;
    }
}
