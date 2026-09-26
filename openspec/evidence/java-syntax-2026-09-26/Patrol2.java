public class Patrol2 {

    // assert 带消息（Java 8 -ea 语义；无 -ea 时被 javac 收成 ifne+new AssertionError）
    static int check(int arg0) {
        assert arg0 > 0 : "bad " + arg0;
        return arg0 * 2;
    }

    // static synchronized 方法（monitor enter on Class）
    static synchronized int bump(int arg0) { return arg0 + 1; }

    // do-while + continue（continue 到闩锁）
    static int dow(int arg0) {
        int n = 0;
        do {
            n++;
            if (n % 2 == 0) continue;
            n += 5;
        } while (n < arg0);
        return n;
    }

    // 字符串 switch + null 选择子
    static int strSwitch(String s) {
        switch (s) {
            case "a": return 1;
            case "b": return 2;
            default: return -1;
        }
    }

    // 条件表达式两臂都有副作用 + 嵌套三元
    static int cond(int arg0) {
        int r = arg0 > 0 ? side(1) : side(2);
        int t = arg0 > 10 ? (arg0 > 100 ? 3 : 2) : 1;
        return r + t;
    }
    static int side(int x) { calls++; return x; }
    static int calls;

    // 位运算混排优先级 + 无符号右移
    static int bits(int a, int b, int c) {
        return (a & b) | (c ^ 0xFF) | (a >>> 3) | (a << 1);
    }

    // 浮点比较（NaN 排序、0.0/-0.0）
    static boolean fp(double a, double b) {
        return a > b || a <= b;
    }

    // long 比较 + long 常量运算
    static int lng(long a) {
        long m = a * 0x1_0000_0000L;
        if (m == -1L) return 1;
        if (m > Long.MAX_VALUE / 2) return 2;
        return (int) (m >>> 32);
    }

    // 嵌套 try + 多捕获（union types）
    static int multiCatch(int arg0) {
        try {
            if (arg0 == 0) throw new NumberFormatException("n");
            if (arg0 == 1) throw new java.nio.file.FileSystemException("f");
            return 0;
        } catch (NumberFormatException | java.nio.file.FileSystemException e) {
            return e.getMessage() == null ? 1 : 2;
        }
    }

    // 增强型 for over 数组与 Iterable，体内 break/continue
    static int foreach(int[] arr, java.util.List<String> list) {
        int n = 0;
        for (int x : arr) { if (x < 0) continue; n += x; }
        for (String s : list) { if (s.isEmpty()) break; n++; }
        return n;
    }
}
