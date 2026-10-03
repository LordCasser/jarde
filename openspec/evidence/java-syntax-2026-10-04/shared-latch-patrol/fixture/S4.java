import java.util.*;
public class S4 {
    // no cast in concat: String keys directly, int values (no boxing cast in chain)
    public static List<String> nestedNoCast(List<String> ks, List<String> vs) {
        List<String> out = new ArrayList<String>();
        for (String k : ks) {
            for (String v : vs) {
                out.add(k + ":" + v);
            }
        }
        return out;
    }
    // cast in concat but single loop
    public static List<String> singleCastConcat(List<Object> xs) {
        List<String> out = new ArrayList<String>();
        for (Object x : xs) { out.add(((String) x) + "!"); }
        return out;
    }
    // nested loop, cast present but concat operands are pre-stored locals
    public static List<String> nestedPreStored(Map<String, List<Integer>> m, String p) {
        List<String> out = new ArrayList<String>();
        for (Map.Entry<String, List<Integer>> e : m.entrySet()) {
            String key = e.getKey();
            if (!key.startsWith(p)) continue;
            for (Integer v : e.getValue()) {
                int iv = v.intValue();
                out.add(key + ":" + iv);
            }
        }
        return out;
    }
    public static void main(String[] a) {
        System.out.println(nestedNoCast(Arrays.asList("k"), Arrays.asList("v")));
        System.out.println(singleCastConcat(Arrays.<Object>asList("c")));
        Map<String, List<Integer>> m = new HashMap<String, List<Integer>>();
        m.put("px", Arrays.asList(3, 5));
        System.out.println(nestedPreStored(m, "p"));
    }
}
