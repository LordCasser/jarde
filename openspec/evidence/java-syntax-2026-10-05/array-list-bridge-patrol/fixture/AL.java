import java.util.*;
public class AL {
    static List<String> bridge(String[] xs){ return Arrays.asList(xs); }                    // 数组↔List 桥
    static String[] back(List<String> l){ return l.toArray(new String[0]); }                  // toArray(new T[0]) 惯用法
    static List<String> frozen(List<String> l){ return Collections.unmodifiableList(l); }     // 不可变包装
    static String firstOr(List<String> l, String dflt){ return l.isEmpty() ? dflt : l.get(0); }   // 空守卫三元
    public static void main(String[] a){ List<String> b = bridge(new String[]{"x","y"}); System.out.println(""+b+"/"+java.util.Arrays.toString(back(b))+"/"+frozen(b).size()+"/"+firstOr(new ArrayList<String>(),"E")+"/"+firstOr(b,"E")); }
}
