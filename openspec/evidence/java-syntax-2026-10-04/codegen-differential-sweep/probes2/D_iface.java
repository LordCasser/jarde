public class D_iface {
    interface I { default int d(){ return 1; } static int s(){ return 2; } int a(); }   // default+static 接口方法
    static class C implements I { public int a(){ return d()+I.s(); } }
    public static void main(String[] x){ System.out.println(new C().a()); }
}
