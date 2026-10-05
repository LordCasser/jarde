public class PC {
    static java.util.List<String> list = new java.util.ArrayList<>();
    static int idx = 0;
    static java.lang.StringBuilder sb = new java.lang.StringBuilder();
    static int counter = 0;
    static int argPos(java.util.List<String> l, int i, String v){ l.set(i, v); return i; }   // 普通形对照
    static int viaArg(){ list.add("a"); list.add("b"); list.set(idx++, "X"); return idx; }    // post-inc 方法实参位
    static int viaReturn(){ return counter++; }                                              // post-inc 返回位
    static String viaChain(){ int t = 5; sb.append("n").append(t++); return sb.toString() + ":" + t; }  // post-inc 链实参位
    public static void main(String[] a){ System.out.println(""+viaArg()+"/"+viaReturn()+"/"+viaReturn()+"/"+viaChain()); }
}
