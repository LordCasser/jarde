public class CS {
    static String join(String[] xs){ return String.join("-", xs); }                        // SB 主锚：标量位 + 数组位
    static void appender(Appendable out) throws java.io.IOException { out.append("x"); }    // Appendable.append(CharSequence)
    static String joining(java.util.List<String> xs){ return xs.stream().collect(java.util.stream.Collectors.joining(",")); }  // java.util.stream 第 5 位点
    static <T extends java.io.Serializable & java.lang.Comparable<T>> T both(T a, T b){ return a.compareTo(b) >= 0 ? a : b; }   // 多重界（Serializable 首界）
    static String useBoth(){ return both("a", "b"); }                                      // Serializable 实参位
    static String same(String s){ return s; }                                              // 同型回答对照（不引入 cast）
    public static void main(String[] a){ System.out.println(join(new String[]{"p","q"})+"/"+joining(java.util.Arrays.asList("a","b","c"))+"/"+useBoth()+"/"+same("s")); }
}
