public class ParentEdge implements Runnable {
    public void run() {}
    public static void consume(Runnable x) {}
    public static void test() { consume(new ParentEdge()); }
}
