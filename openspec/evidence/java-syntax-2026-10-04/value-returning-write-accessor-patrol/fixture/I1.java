import java.util.*;
public class I1 {
    private static final Map<String,Integer> REG = new HashMap<String,Integer>();
    private int counter;
    // static initializer
    static { REG.put("a", 1); REG.put("b", 2); }
    // instance initializer
    { counter = 10; }
    I1() { counter += 1; }
    // non-static inner class reading outer field (synthetic accessor)
    class Inner {
        int bump() { counter += 5; return counter; }
        int read() { return counter; }
    }
    // static nested class
    static class Nested { static int twice(int x) { return x * 2; } }
    // private member accessed from inner class -> synthetic accessor
    private int secret = 7;
    class Reader { int get() { return secret; } void set(int v) { secret = v; } }
    public static void main(String[] a) {
        System.out.println(REG.get("a") + REG.get("b"));
        I1 o = new I1();
        System.out.println(o.counter);
        Inner in = o.new Inner();
        System.out.println(in.bump() + "/" + in.read());
        System.out.println(Nested.twice(21));
        Reader r = o.new Reader();
        System.out.println(r.get());
        r.set(9);
        System.out.println(o.secret + "/" + r.get());
    }
}
