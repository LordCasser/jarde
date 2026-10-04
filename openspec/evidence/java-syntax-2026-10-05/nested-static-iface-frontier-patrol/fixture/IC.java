public class IC {
    interface Op { int apply(int x); }
    static class Outer {
        static class Inner { static int stat = 5; int inst = 6; static int statM(){ return 50; } int instM(){ return 60; } }
    }
    static int viaStatic(){ return Outer.Inner.stat + Outer.Inner.statM(); }          // 静态嵌套：静态成员访问
    static int viaInst(){ Outer.Inner i = new Outer.Inner(); return i.inst + i.instM(); } // 静态嵌套：实例化+实例成员
    static int viaIface(Op o){ return o.apply(3); }                                    // 接口参数多态
    static int lambda(){ return viaIface(x -> x * 7); }                                // lambda 入接口参数
    public static void main(String[] a){
        System.out.println(viaStatic()+"/"+viaInst()+"/"+lambda());
    }
}
