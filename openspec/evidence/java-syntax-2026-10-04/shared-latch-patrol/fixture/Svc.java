import java.util.*;
public class Svc {
    private final Map<String, List<Integer>> cache = new HashMap<String, List<Integer>>();
    private int hits;
    enum Mode { FAST, SAFE }
    static class Entry {
        final String key; final int val;
        Entry(String k, int v) { key = k; val = v; }
        String render() { return key + "=" + val; }
    }
    public List<String> lookup(String prefix, Mode mode) {
        List<String> out = new ArrayList<String>();
        for (Map.Entry<String, List<Integer>> e : cache.entrySet()) {
            if (!e.getKey().startsWith(prefix)) continue;
            for (Integer v : e.getValue()) {
                if (mode == Mode.SAFE && v < 0) break;
                out.add(e.getKey() + ":" + v);
            }
        }
        hits++;
        return out;
    }
    public String summarize() {
        StringBuilder sb = new StringBuilder();
        List<Entry> es = new ArrayList<Entry>();
        es.add(new Entry("a", 1));
        es.add(new Entry("b", 2));
        for (Entry e : es) sb.append(e.render()).append(';');
        return sb.append("hits=").append(hits).toString();
    }
    public static void main(String[] a) {
        Svc s = new Svc();
        s.cache.put("px", Arrays.asList(3, -1, 5));
        System.out.println(s.lookup("p", Mode.SAFE));
        System.out.println(s.summarize());
        System.out.println(s.lookup("z", Mode.FAST).isEmpty());
    }
}
