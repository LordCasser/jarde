import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
public class Z1<T extends Comparable<T>> {
    private List<T> items = new ArrayList<T>();
    private Map<String, List<T>> index = new HashMap<String, List<T>>();
    public void add(T item) { items.add(item); }
    public T first() { return items.get(0); }
    public <R> List<R> map(java.util.function.Function<? super T, ? extends R> f) {
        List<R> out = new ArrayList<R>();
        for (T t : items) { out.add(f.apply(t)); }
        return out;
    }
    static class Box<U> { U value; }
    Box<String> named() { Box<String> b = new Box<String>(); b.value = "x"; return b; }
    public static void main(String[] a) {
        Z1<String> z = new Z1<String>();
        z.add("b"); z.add("a");
        System.out.println(z.first());
        System.out.println(z.map(s -> s.length()));
        System.out.println(z.named().value);
    }
}
