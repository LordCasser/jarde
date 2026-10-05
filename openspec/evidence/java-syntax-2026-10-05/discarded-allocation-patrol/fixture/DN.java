public class DN {
    static class N { void hi(){ System.out.println("hi"); } }
    static void discarded(){ new N(); System.out.println("after"); }        // new 语句丢弃结果
    static void used(){ N n = new N(); n.hi(); }                             // 赋值使用（对照）
    static void chained(){ new N().hi(); }                                   // 链式调用（对照）
    static void argUse(){ takes(new N()); }                                  // 实参使用（对照）
    static void takes(N n){ n.hi(); }
    public static void main(String[] a){ discarded(); used(); chained(); argUse(); }
}
