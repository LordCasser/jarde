public class NI {
    class Inner {                                   // 非静态内部类
        int v;
        Inner(int v){ this.v = v; }
        int outerTag(){ return tag * 2; }           // 读外部实例字段（隐式 Outer.this）
        NI outerRef(){ return NI.this; }            // Qualified this
    }
    int tag = 5;
    Inner make(int v){ return new Inner(v); }       // 外部类内 new（隐式 this 作首参）
    static Inner externalMake(NI outer, int v){ return outer.new Inner(v); }   // outer.new（限定外部实例 new）
    public static void main(String[] a){ NI n = new NI(); System.out.println(""+n.make(3).v+"/"+n.make(2).outerTag()+"/"+externalMake(n, 1).outerRef().tag+"/"+(externalMake(n,1).outerRef() == n)); }
}
