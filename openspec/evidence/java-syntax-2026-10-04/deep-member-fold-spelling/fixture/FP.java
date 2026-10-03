public class FP {
    static class Mid { static class Leaf { } }
    // 不含类字面量：经实例的 getClass() 做结构反射
    public static String viaInstance() { return new Mid.Leaf().getClass().getSimpleName(); }
    public static String viaInstanceChain() { return new Mid.Leaf().getClass().getEnclosingClass().getSimpleName(); }
    public static void main(String[] a) {
        System.out.println(viaInstance());
        System.out.println(viaInstanceChain());
    }
}
