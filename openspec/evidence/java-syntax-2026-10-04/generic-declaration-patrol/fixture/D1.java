import java.util.*;
public class D1 {
    static List<String> fieldList = new ArrayList<>();
    Map<String, List<Integer>> map = new HashMap<>();
    public static List<String> makeList() { return new ArrayList<>(); }
    public static Map<String, Set<Long>> makeNested() { return new HashMap<>(); }
    public static void consume(List<String> l) { }
    public static void callSite() { consume(new ArrayList<>()); }
    public static List<String> localVar() { List<String> x = new LinkedList<>(); x.add("a"); return x; }
    public static <T> List<T> generic(T v) { List<T> out = new ArrayList<>(); out.add(v); return out; }
    public static void main(String[] a) {
        System.out.println(makeList().size());
        System.out.println(makeNested().isEmpty());
        callSite();
        System.out.println(localVar());
        System.out.println(generic("z"));
        System.out.println(fieldList.size());
        System.out.println(new D1().map.isEmpty());
    }
}
