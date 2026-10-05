public class VR {
    static int sum(int... xs){ int s = 0; for(int x : xs){ s += x; } return s; }      // varargs 定义
    static int fwd(int... xs){ return sum(xs); }                                       // varargs 转发（args 直接传）
    static int spread(){ int[] arr = {1, 2, 3}; return sum(arr); }                     // 数组展开调用
    static int mixed(int a, String... ss){ int n = a; for(String s : ss){ n += s.length(); } return n; }  // 固定+varargs
    static String[] make(){ return new String[]{"a", "bb", "ccc"}; }
    public static void main(String[] a){ System.out.println(""+sum(1,2,3)+"/"+fwd(4,5)+"/"+spread()+"/"+mixed(1,"x","yy")+"/"+mixed(2, make())+"/"+mixed(3)); }
}
