public class SG {
    static class Hold<T> { Object v; Hold(Object v){ this.v = v; } }
    static Hold<String> f1 = new Hold<>("a");
    static Hold<String> f2 = new Hold<String>("b");
    Hold<Integer> f3 = new Hold<>(5);
    public static void main(String[] a){ SG m = new SG(); System.out.println("" + SG.f1.v + SG.f2.v + m.f3.v); }
}
