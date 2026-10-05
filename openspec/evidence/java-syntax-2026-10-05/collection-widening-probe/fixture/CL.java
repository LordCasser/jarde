import java.util.*;
public class CL {
    static boolean retain(List<String> a, Set<String> b){ return a.retainAll(b); }            // List.retainAll(Set)
    static boolean contains(List<String> a, Set<String> b){ return a.containsAll(b); }        // containsAll
    static boolean disjoint(List<String> a, Set<String> b){ return Collections.disjoint(a, b); }  // 静态泛型
    static <T extends Collection<?>> int size(T c){ return c.size(); }                        // 泛型方法擦除到 Collection
    static int sizeCall(){ return size(new ArrayList<String>(Arrays.asList("x"))); }                // 泛型调用点（ArrayList presents Collection）
    public static void main(String[] a){
        List<String> la = new ArrayList<String>(Arrays.asList("a", "b"));
        Set<String> sa = new HashSet<String>(Arrays.asList("a"));
        List<String> one = new ArrayList<String>(Arrays.asList("a"));
        System.out.println(""+retain(la, sa)+"/"+contains(one, sa)+"/"+disjoint(Arrays.asList("p"), sa)+"/"+sizeCall());
    }
}
