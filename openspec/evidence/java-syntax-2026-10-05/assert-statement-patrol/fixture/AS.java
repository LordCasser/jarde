public class AS {
    static int simple(int x){ assert x > 0 : "pos expected"; return x * 2; }        // assert+message
    static int bare(int x){ assert x > 0; return x; }                                 // assert 无 message
    static boolean en(){ return AS.class.desiredAssertionStatus(); }                 // 断言启用查询
    public static void main(String[] a){ System.out.println(""+bare(3)+"/"+en()); } // 不触发断言路径
}
