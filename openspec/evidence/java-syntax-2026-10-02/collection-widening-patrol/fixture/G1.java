import java.util.ArrayList;
import java.util.List;
public class G1 {
    static <T> T pick(T a, T b) { return a; }
    static <T extends Comparable<T>> T max(List<T> xs) {
        T best = xs.get(0);
        for (T x : xs) { if (x.compareTo(best) > 0) best = x; }
        return best;
    }
    public static String use() {
        List<String> xs = new ArrayList<String>();
        xs.add("pear"); xs.add("apple"); xs.add("zeta");
        String m = max(xs);
        String p = pick("a", "b");
        return m + ":" + p;
    }
    public static int autoboxLoop() {
        Integer sum = 0;
        for (Integer i = 1; i <= 3; i++) { sum += i; }
        return sum;
    }
    public static void main(String[] a) {
        System.out.println(use());
        System.out.println(autoboxLoop());
    }
}
