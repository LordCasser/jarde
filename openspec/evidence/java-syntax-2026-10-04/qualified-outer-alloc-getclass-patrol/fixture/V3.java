public class V3 {
    private int base = 4;
    class Inner { private int tag; Inner(int t){tag=t;} int total(){return base+tag;} }
    static class Stat { int use(V3 outer) { return outer.new Inner(9).total(); } }
    Inner make(int t) { return new Inner(t); }
    public static void main(String[] a){ System.out.println(new Stat().use(new V3())); }
}
