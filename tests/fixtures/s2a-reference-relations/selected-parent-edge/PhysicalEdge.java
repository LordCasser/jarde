public class PhysicalEdge implements EdgeTarget {
    public static void consume(EdgeTarget x) {}
    public static void test() { consume(new PhysicalEdge()); }
}
