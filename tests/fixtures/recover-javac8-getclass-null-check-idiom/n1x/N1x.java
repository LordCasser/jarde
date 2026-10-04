public class N1x {
    private int base = 4;
    class Inner {
        private int tag;
        Inner(int t) { this.tag = t; }
        int total() { return tag + base; }
    }
    static class Stat {
        int m() { return 1; }
        int use(N1x outer) { return outer.new Inner(9).total(); }
    }
    Inner make(int t) { return new Inner(t); }
    static int useStatic() { return new N1x().make(6).total(); }
    public static int statUse(N1x outer) { return new Stat().use(outer); }
}
