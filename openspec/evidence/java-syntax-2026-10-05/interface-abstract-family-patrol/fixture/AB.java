public class AB {
    interface A { int a(); }
    interface B extends A { int b(); }                    // 接口继承
    abstract static class Part implements B { public int a(){ return 1; } public abstract int b(); } // 抽象类部分实现
    static class Full extends Part { public int b(){ return 2; } }
    interface Consts { int X = 10; long L = 20L; String S = "cs"; }  // 接口常量（隐式 public static final）
    static int useAll(B b){ return b.a()+b.b(); }
    static int useConsts(){ return Consts.X + (int)Consts.L + Consts.S.length(); }
    public static void main(String[] a){ System.out.println(""+useAll(new Full())+"/"+useConsts()); }
}
