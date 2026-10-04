import java.util.*;
import java.util.function.*;
public class M1 {
    static class Item { String name; Item(String n){name=n;} String get(){return name;} }
    static String stat(String s) { return s + "!"; }
    private String inst(String s) { return s + "?"; }
    public static void main(String[] a) {
        List<String> xs = new ArrayList<>(Arrays.asList("a","b"));
        // 1. 静态方法引用
        xs.forEach(M1::stat);
        // 2. 特定实例的方法引用
        M1 m = new M1();
        Function<String,String> f = m::inst;
        System.out.println(f.apply("x"));
        // 3. 任意对象的实例方法引用
        Function<String,Integer> g = String::length;
        System.out.println(g.apply("hello"));
        // 4. 构造器引用
        Supplier<Item> s = () -> new Item("lit");
        Function<String,Item> c = Item::new;
        System.out.println(c.apply("made").get());
        // 5. 数组构造器引用
        Function<Integer,String[]> arr = String[]::new;
        System.out.println(arr.apply(3).length);
        // 6. Stream 链
        long n = xs.stream().filter(t -> t.startsWith("a")).map(M1::stat).count();
        System.out.println("count=" + n);
    }
}
