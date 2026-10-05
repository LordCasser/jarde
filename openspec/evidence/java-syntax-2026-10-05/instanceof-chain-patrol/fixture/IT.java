public class IT {
    static class A { String id(){ return "A"; } }
    static class B extends A { String id(){ return "B"; } int extra(){ return 1; } }
    static class C extends A { String id(){ return "C"; } }
    static String dispatch(A x){                                     // instanceof 链类型分派（子类先测——顺序语义关键）
        if(x instanceof C){ return "c:" + ((C) x).id(); }
        else if(x instanceof B){ return "b:" + ((B) x).extra(); }
        else if(x instanceof A){ return "a:" + x.id(); }
        return "none";
    }
    static boolean negTest(Object o){ return o instanceof A; }        // Object 位测（负形 false）
    static String guard(A x){                                        // instanceof 后直接用（无显式 cast——javac 仍发 checkcast? no: 调用点 x.id() 已是 A 方法）
        if(x instanceof B){ return "guarded:" + x.id(); }
        return "plain:" + x.id();
    }
    public static void main(String[] a){ System.out.println(""+dispatch(new C())+"/"+dispatch(new B())+"/"+dispatch(new A())+"/"+negTest(new Object())+"/"+negTest(new A())+"/"+guard(new B())); }
}
