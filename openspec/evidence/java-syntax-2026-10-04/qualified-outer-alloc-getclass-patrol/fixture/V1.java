public class V1 {
    private int base = 4;
    class Inner { private int tag; Inner(int t){tag=t;} int total(){return base+tag;} }
    static class Stat { int use(V1 outer) { return outer.new Inner(9).total(); } }
    public static void main(String[] a){ System.out.println(new Stat().use(new V1())); }
}
