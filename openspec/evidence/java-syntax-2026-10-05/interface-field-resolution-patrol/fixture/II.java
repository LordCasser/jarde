public class II {
    interface A { int V = 1; }                       // 接口常量（隐式 public static final）
    interface B { int V = 2; }
    interface C extends A { }                        // 继承接口常量
    static class UseBoth implements A, B {           // 双接口同名字段冲突
        int a(){ return A.V; }
        int b(){ return B.V; }                       // 必须限定
    }
    static class UseC implements C { int c(){ return C.V; } }  // 经继承链访问
    static class Shadows implements A { int V = 9; int own(){ return V; } int iface(){ return A.V; } } // 自字段遮蔽接口常量
    public static void main(String[] a){ UseBoth u = new UseBoth(); Shadows s = new Shadows(); System.out.println(""+u.a()+"/"+u.b()+"/"+new UseC().c()+"/"+s.own()+"/"+s.iface()); }
}
