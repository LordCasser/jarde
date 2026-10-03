import java.util.*;
public class S3 {
    public static List<String> nestedBreak(Map<String, List<Integer>> cache, String prefix) {
        List<String> out = new ArrayList<String>();
        for (Map.Entry<String, List<Integer>> e : cache.entrySet()) {
            if (!e.getKey().startsWith(prefix)) continue;
            for (Integer v : e.getValue()) {
                if (v < 0) break;
                out.add(e.getKey() + ":" + v);
            }
        }
        return out;
    }
    public static List<String> nestedNoJump(Map<String, List<Integer>> cache, String prefix) {
        List<String> out = new ArrayList<String>();
        for (Map.Entry<String, List<Integer>> e : cache.entrySet()) {
            if (!e.getKey().startsWith(prefix)) continue;
            for (Integer v : e.getValue()) {
                out.add(e.getKey() + ":" + v);
            }
        }
        return out;
    }
    public static List<String> singleLoopConcat(List<String> xs) {
        List<String> out = new ArrayList<String>();
        for (String x : xs) { out.add(x + "!"); }
        return out;
    }
    public static void main(String[] a) {
        Map<String, List<Integer>> m = new HashMap<String, List<Integer>>();
        m.put("px", Arrays.asList(3, -1, 5));
        System.out.println(nestedBreak(m, "p"));
        System.out.println(nestedNoJump(m, "p"));
        System.out.println(singleLoopConcat(Arrays.asList("a", "b")));
    }
}
