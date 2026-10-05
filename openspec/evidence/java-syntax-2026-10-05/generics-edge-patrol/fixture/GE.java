public class GE {
    static class Box<T> { T v; void set(T t){ this.v = t; } T get(){ return v; } }
    static class Pair<A, B> extends Box<A> { B w; }                    // 子类加参数
    static <T extends java.io.Serializable & java.lang.Comparable<T>> T both(T a, T b){ return a.compareTo(b) >= 0 ? a : b; }  // 多重界
    static String useBoth(){ return both("a", "b"); }
    static java.util.List<? extends Number> up(){ return java.util.Arrays.asList(1, 2L); }   // 通配符返回
    static int readUp(java.util.List<? extends Number> l){ return l.get(0).intValue(); }      // 通配符读
    static void writeDown(java.util.List<? super Integer> l){ l.add(1); }                     // 通配符写（下界）
    public static void main(String[] a){ System.out.println(""+useBoth()+"/"+readUp(up())+"/"+downCheck()); }
    static int downCheck(){ java.util.List<java.lang.Object> l = new java.util.ArrayList<>(); writeDown(l); return ((Integer) l.get(0)); }
}
