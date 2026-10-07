public class BNX {
    static <T extends Number> T viaDecimal(T a, T b) { return a.doubleValue() >= b.doubleValue() ? a : b; }
    static <T extends Number> T viaAtomic(T a, T b) { return a.doubleValue() >= b.doubleValue() ? a : b; }

    public static void main(String[] a) {
        System.out.println(viaDecimal(new java.math.BigDecimal("1"), new java.math.BigDecimal("2")));
        System.out.println(viaAtomic(new java.util.concurrent.atomic.AtomicInteger(1), new java.util.concurrent.atomic.AtomicInteger(2)));
    }
}
