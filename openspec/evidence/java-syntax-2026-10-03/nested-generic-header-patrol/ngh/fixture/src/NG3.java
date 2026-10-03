public class NG3 {
    static class Num<N extends Number> { N n; N get() { return n; } }
    public static void main(String[] a) {
        Num<java.lang.Integer> num = new Num<java.lang.Integer>();
        num.n = 7;
        System.out.println(num.get());
    }
}
