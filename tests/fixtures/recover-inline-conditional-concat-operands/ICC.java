public class ICC {
    class Inner {
        int v;
        Inner(int v){ this.v = v; }
        int outerTag(){ return tag * 2; }
        ICC outerRef(){ return ICC.this; }
    }
    int tag = 5;
    Inner make(int v){ return new Inner(v); }
    static Inner externalMake(ICC outer, int v){ return outer.new Inner(v); }
    public static void main(String[] a){
        ICC n = new ICC();
        System.out.println(""+n.make(3).v+"/"+n.make(2).outerTag()+"/"+externalMake(n, 1).outerRef().tag+"/"+(externalMake(n,1).outerRef() == n));
    }
}
