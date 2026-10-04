import java.util.*;
import java.util.function.*;
import java.io.*;

public class C1 {
    // 泛型嵌套 + 静态字段 + diamond
    static final Map<String, List<Integer>> INDEX = new HashMap<>();
    // 泛型方法 + 通配符
    static <T extends Comparable<? super T>> T maxOf(List<? extends T> xs) {
        T best = null;
        for (T x : xs) if (best == null || x.compareTo(best) > 0) best = x;
        return best;
    }
    // 枚举 + 常量专属体 + 字段
    enum Mode { FAST("f"), SLOW("s") { @Override public String tag() { return "S" + super.tag(); } };
        private final String t; Mode(String t){this.t=t;} public String tag(){return t;} }
    // 匿名类 + 捕获 + super 实参（混合形）
    static Runnable mk(final String label) {
        final int n = label.length();
        return new Runnable() { public void run() { System.out.println(label + n); } };
    }
    // try-with-resources + 多资源 + 异常
    static int count(String p) {
        try (BufferedReader r = new BufferedReader(new StringReader(p))) {
            String l = r.readLine();
            return l == null ? 0 : l.length();
        } catch (IOException e) { return -1; }
    }
    // stream + lambda 多语句体 + 方法引用 + 装箱
    static List<String> chain(List<Integer> xs) {
        List<String> out = new ArrayList<>();
        xs.stream().filter(i -> { int d = i * 2; return d > 4; })
                  .map(C1::show).sorted().forEach(out::add);
        return out;
    }
    static String show(int i) { return "n" + i; }
    // switch on String + fallthrough + default
    static String pick(String k) {
        switch (k) { case "a": case "b": return "AB"; case "c": return "C"; default: return "?"; }
    }
    // 三元 + 位运算 + 自增 + 复合赋值
    static int arith(int a) { int b = a > 0 ? a : -a; b |= 3; b <<= 1; b += 7; return b++; }

    public static void main(String[] args) throws Exception {
        INDEX.put("k", Arrays.asList(3, 1, 2));
        System.out.println(maxOf(INDEX.get("k")));
        System.out.println(Mode.SLOW.tag() + "/" + Mode.FAST.tag());
        mk("xy").run();
        System.out.println(count("hello"));
        System.out.println(chain(Arrays.asList(1, 2, 3, 4)));
        System.out.println(pick("b") + pick("c") + pick("z"));
        System.out.println(arith(-5));
    }
}
