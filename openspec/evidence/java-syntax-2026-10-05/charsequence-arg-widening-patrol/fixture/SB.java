public class SB {
    static String acc(java.util.List<String> xs){ StringBuilder b = new StringBuilder(); for(String x : xs){ b.append(x).append(','); } return b.toString(); } // 循环累积链式
    static String cond(java.util.List<String> xs){ String r = ""; for(String x : xs){ if(x.length() > 1) r = r + x; } return r; } // String 循环重赋值（每轮新 builder）
    static int[] copy(int[] a){ int[] b = new int[a.length]; System.arraycopy(a, 0, b, 0, a.length); return b; } // arraycopy
    static int[] of(int[] a){ return java.util.Arrays.copyOf(a, a.length + 1); }           // copyOf（含扩容）
    static String join(String[] xs){ return String.join("-", xs); }                         // String.join (JDK8)
    static int spread(int... xs){ int s = 0; for(int x : xs) s += x; return s; }           // varargs 收集
    static int call(){ return spread(1, 2, 3); }                                            // varargs 展开（数组直传）
    public static void main(String[] a){ System.out.println(acc(java.util.Arrays.asList("a","b"))+"/"+cond(java.util.Arrays.asList("x","yy"))+"/"+java.util.Arrays.toString(copy(new int[]{1,2}))+"/"+java.util.Arrays.toString(of(new int[]{1}))+"/"+join(new String[]{"p","q"})+"/"+spread(new int[]{4,5})+"/"+call()); }
}
