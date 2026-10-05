import java.util.*;
public class MC {
    static Map<String,Integer> cache = new HashMap<>();
    static int memoGet(String k){                                        // get + null guard + put（memoization 惯用法）
        Integer v = cache.get(k);
        if(v == null){ v = k.length(); cache.put(k, v); }
        return v;
    }
    static String containsKey(String k, Map<String,String> m){           // containsKey 消歧（null 值合法）
        if(m.containsKey(k)){ return m.get(k); }
        return "absent";
    }
    static int computeIfAbsent(String k, Map<String,Integer> m){         // computeIfAbsent lambda
        return m.computeIfAbsent(k, s -> s.length() * 2);
    }
    static String getOrDefault(String k, Map<String,String> m){          // getOrDefault
        return m.getOrDefault(k, "dflt");
    }
    public static void main(String[] a){ System.out.println(""+memoGet("abc")+"/"+memoGet("abc")+"/"+containsKey("x", new HashMap<>(Collections.singletonMap("x","hit")))+"/"+containsKey("y", new HashMap<>())+"/"+computeIfAbsent("hi", new HashMap<>())+"/"+getOrDefault("z", new HashMap<>())); }
}
