public class NI2 {
    class Inner {
        int v;
        Inner(int v){ this.v = v; }
        int outerTag(){ return tag * 2; }
        NI2 outerRef(){ return NI2.this; }
    }
    int tag = 5;
    Inner make(int v){ return new Inner(v); }
    static Inner externalMake(NI2 outer, int v){ return outer.new Inner(v); }
    public static void main(String[] a){ NI2 n = new NI2(); System.out.println(""+n.make(3).v+"/"+n.make(2).outerTag()+"/"+externalMake(n, 1).outerRef().tag+"/"+n.tag); }
}
