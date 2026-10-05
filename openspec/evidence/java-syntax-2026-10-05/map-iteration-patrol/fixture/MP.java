public class MP {
    static java.util.Map<String,Integer> m = new java.util.HashMap<>();
    static { m.put("a", 1); m.put("b", 2); }
    static int viaEntry(){ int s = 0; for(java.util.Map.Entry<String,Integer> e : m.entrySet()){ s += e.getKey().length() + e.getValue(); } return s; }   // entrySet 遍历（最常见）
    static int viaKeySet(){ int s = 0; for(String k : m.keySet()){ s += k.length() + m.get(k); } return s; }                                            // keySet+get
    static int viaValues(){ int s = 0; for(int v : m.values()){ s += v; } return s; }                                                                    // values
    static int viaEntryNoGen(){ int s = 0; for(java.util.Map.Entry e : m.entrySet()){ s += ((String) e.getKey()).length() + (Integer) e.getValue(); } return s; }  // raw entrySet
    public static void main(String[] a){ System.out.println(""+viaEntry()+"/"+viaKeySet()+"/"+viaValues()+"/"+viaEntryNoGen()); }
}
