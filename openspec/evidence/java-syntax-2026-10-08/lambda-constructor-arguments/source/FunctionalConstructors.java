import java.util.PriorityQueue;
import java.util.concurrent.FutureTask;
public class FunctionalConstructors {
    public static int trace;
    public static Thread runnable() { return new Thread(() -> trace += 3); }
    public static Thread captured(int n) { return new Thread(() -> trace += n); }
    public static PriorityQueue<Integer> comparator() { return new PriorityQueue<>((a,b) -> b-a); }
    public static PriorityQueue<Integer> reference() { return new PriorityQueue<>(FunctionalConstructors::compare); }
    public static int compare(Integer a, Integer b) { return b-a; }
    public static FutureTask<Integer> callable() { return new FutureTask<>(() -> 7); }
    public static IntBox primitive(int n) { return new IntBox(x -> n+x); }
    public static IntBox primitiveReference() { return new IntBox(FunctionalConstructors::twice); }
    public static int twice(int x) { return x*2; }
    public static String name() { trace = trace*10+1; return "worker"; }
    public static Thread ordered(int n) { return new Thread(() -> trace = trace*10+n, name()); }
    public static Thread overload() { return new Thread((Runnable) () -> trace += 2); }
}
