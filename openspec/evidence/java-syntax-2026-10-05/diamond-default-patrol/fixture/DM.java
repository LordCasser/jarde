public class DM {
    interface A { default String n(){ return "A"; } }
    interface B { default String n(){ return "B"; } }
    static class Both implements A, B {                 // diamond 冲突——必须显式 override
        @Override public String n(){ return "A:"+A.super.n(); }
    }
    static class Simple implements A {}                 // 单继承 default（隐式）
    static class Re extends Simple {                    // 二级继承
        @Override public String n(){ return "S+"+super.n(); }
    }
    interface C extends A { @Override default String n(){ return "C"; } } // 接口再抽象化 override
    static class ViaC implements C {}
    public static void main(String[] a){ System.out.println(""+new Both().n()+"/"+new Simple().n()+"/"+new Re().n()+"/"+new ViaC().n()); }
}
