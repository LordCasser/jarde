public class N1 {
    private int base = 4;
    class Inner {
        private int tag;
        Inner(int t) { this.tag = t; }
        int total() { return tag + base; }
    }
    static class Stat {
        int m() { return 1; }
        int use(N1 outer) { return outer.new Inner(9).total(); }
    }
    Inner make(int t) { return new Inner(t); }
    static int useStatic() { return new N1().make(6).total(); }
    public static void main(String[] a) {
        System.out.println(useStatic());
        System.out.println(new N1().new Inner(3).total());
        System.out.println(new Stat().use(new N1()));
    }
}
