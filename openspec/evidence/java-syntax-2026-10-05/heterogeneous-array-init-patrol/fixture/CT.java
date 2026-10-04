public class CT {
    static String io(Object o){ if(o instanceof String){ String s = (String) o; return s.toUpperCase(); } return "?"; }   // instanceof+cast 模式
    static String io2(Object o){ return (o instanceof String) ? ((String) o).trim() : "?"; }                                  // 三元中的双重 cast
    static Object up(){ Object l = java.util.Arrays.asList("a","b"); return l; }                                              // 装箱上转型赋值
    static Number cov(){ java.util.List<? extends Number> ln = java.util.Arrays.asList(1, 2L); return ln.get(0); }            // 通配符上界
    static int prim(){ Object b = true ? Integer.valueOf(3) : Double.valueOf(4); return ((Number) b).intValue(); }            // 装箱汇合+cast
    public static void main(String[] a){ System.out.println(io("x")+"/"+io2(" y ")+"/"+up()+"/"+cov()+"/"+prim()); }
}
