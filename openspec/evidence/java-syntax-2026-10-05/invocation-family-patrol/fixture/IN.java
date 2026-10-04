public class IN {
    static class Base { int f(){ return 1; } }
    static class Sub extends Base { @Override int f(){ return 2; } Base asBase(){ return this; } }
    static int virt(Base b){ return b.f(); }                          // 虚分派（调用点类型 Base、运行 Sub）
    static int invspec(){ java.util.List<String> l = new java.util.ArrayList<>(); l.add("x"); return l.size(); } // invokeinterface 常见链
    static int stat(){ return Integer.parseInt("7"); }                // invokestatic
    static int specInv(){ java.util.List<String> l = java.util.Arrays.asList("a","b"); return l.size(); } // 不可变实现上的接口调用
    interface I { default int d(){ return 5; } int v(); }
    static int defImp(){ I i = new I(){ public int v(){ return d()+1; } }; return i.v(); } // default 方法被匿名类消费
    public static void main(String[] a){ Sub s = new Sub(); System.out.println(""+virt(s.asBase())+"/"+invspec()+"/"+stat()+"/"+specInv()+"/"+defImp()); }
}
