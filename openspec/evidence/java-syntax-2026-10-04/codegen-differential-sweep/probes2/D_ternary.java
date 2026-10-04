public class D_ternary {
    static int f(int a,int b,int c){ return a>b ? (b>c ? b : c) : (a>c ? a : c); }  // 嵌套三元
    static String g(Object o){ return o==null ? "null" : o.toString(); }
    public static void main(String[] x){ System.out.println(f(1,2,3)+g(null)+g("s")); }
}
