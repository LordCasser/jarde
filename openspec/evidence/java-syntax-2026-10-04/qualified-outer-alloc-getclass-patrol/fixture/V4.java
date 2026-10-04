public class V4 {
    private int base = 4;
    class Inner { private int tag; Inner(int t){tag=t;} int total(){return base+tag;} }
    static class Stat { int m(){return 1;} int use(V4 outer) { return outer.new Inner(9).total(); } }
    public static void main(String[] a){ System.out.println(new Stat().use(new V4())); }
}
