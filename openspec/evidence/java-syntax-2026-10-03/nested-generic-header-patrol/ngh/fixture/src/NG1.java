public class NG1 {
    class Inner<U> { U value; U get() { return value; } }
    public static void main(String[] a) {
        Inner<String> inner = new NG1().new Inner<String>();
        inner.value = "x";
        System.out.println(inner.get());
    }
}
