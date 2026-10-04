public class V2 {
    private int base = 4;
    class Inner { private int tag; Inner(int t){tag=t;} int total(){return base+tag;} }
    static class Stat { int use(V2 outer) { return outer.new Inner(9).total(); }
                        int loc(V2 outer) { V2.Inner l = outer.new Inner(9); return l.total(); } }
    public static void main(String[] a){ System.out.println(new Stat().use(new V2())); }
}
