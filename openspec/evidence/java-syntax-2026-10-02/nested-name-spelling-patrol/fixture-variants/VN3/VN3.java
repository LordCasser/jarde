public class VN3 {
    interface Op { int apply(int x); }
    static Op capture(final int base) {
        return new Op() { public int apply(int x) { return x + base; } };
    }
    public static void main(String[] a) {
        System.out.println(capture(5).apply(37));
    }
}
