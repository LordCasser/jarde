public class RP {
    // 直属成员带泛型 Signature -> 折叠被拒（"member fold member has a class Signature projection"）
    static class Box<T> { T value; }
    // 方法体对该成员做结构反射消费（getSimpleName）
    public static String simpleName() { return Box.class.getSimpleName(); }
    public static String viaGetName() { return Box.class.getName(); }
    public static void main(String[] a) {
        System.out.println(simpleName());
        System.out.println(viaGetName());
    }
}
