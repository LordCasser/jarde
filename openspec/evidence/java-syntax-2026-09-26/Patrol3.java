import java.util.Arrays;

public class Patrol3 {

    // 方法内局部类：捕获参数与局部、修改后的捕获（合成捕获字段）
    interface Op { int apply(int x); }

    static Op makeOp(int factor) {
        int offset = 10;
        class Scaled implements Op {
            @Override public int apply(int x) { return x * factor + offset; }
        }
        return new Scaled();
    }

    // varargs：声明、增强 for 遍历、透传数组、泛型 varargs
    static int sum(int... xs) {
        int n = 0;
        for (int x : xs) n += x;
        return n;
    }

    static int pass(int first, int... rest) { return first + sum(rest); }

    static <T> int count(T[] items, T probe) {
        int n = 0;
        for (T t : items) if (t.equals(probe)) n++;
        return n;
    }

    // instanceof 数组类型 + 数组协变
    static int probe(Object o) {
        if (o instanceof int[]) {
            int[] a = (int[]) o;
            return a.length;
        }
        if (o instanceof String[]) return ((String[]) o).length * 2;
        return -1;
    }

    // 实例初始化块（跨构造器共享）+ 双构造器
    int base;
    { base = 5; }

    Patrol3() { base += 1; }
    Patrol3(int x) { base += x; }

    int get() { return base; }

    // 多维数组元素复合赋值
    static int grid(int[][] g, int i, int j) {
        g[i][j] += 7;
        return g[i][j];
    }

    // 数组协变写错误类型（ArrayStoreException 路径只是声明，不触发）
    static Object[] alias() { return new String[] { "a", "b" }; }

    static String read(Object[] os, int i) { return (String) os[i]; }
}
