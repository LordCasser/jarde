import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
public class W1 {
    static double sumExt(List<? extends Number> xs) {
        double s = 0;
        for (Number n : xs) { s += n.doubleValue(); }
        return s;
    }
    static void addSuper(List<? super Integer> xs, Integer v) { xs.add(v); }
    static String nameOf(Class<?> c) { return c.getSimpleName(); }
    public static String use() {
        List<Integer> ints = new ArrayList<Integer>(Arrays.asList(1, 2, 3));
        List<Object> objs = new ArrayList<Object>();
        addSuper(objs, 7);
        return sumExt(ints) + ":" + objs.get(0) + ":" + nameOf(W1.class);
    }
    public static void main(String[] a) { System.out.println(use()); }
}
