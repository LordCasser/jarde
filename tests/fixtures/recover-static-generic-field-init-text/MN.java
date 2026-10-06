public class MN {
    static class Hold<T> { T v; Hold(T v){ this.v = v; } }
    static Hold<String> f1 = new Hold<>("a");        // 菱形
    static Hold<String> f2 = new Hold<String>("b");  // 显式类型实参
    Hold<Integer> f3 = new Hold<>(5);                // 实例字段菱形
    public static void main(String[] a){ MN m = new MN(); System.out.println(MN.f1.v+MN.f2.v+m.f3.v); }
}
