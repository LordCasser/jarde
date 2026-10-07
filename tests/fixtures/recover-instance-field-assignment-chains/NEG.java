public class NEG {
    int a, b, c;
    static NEG o1 = new NEG();
    static NEG o2 = new NEG();
    void cross() { o1.a = o2.b = 5; }
    int expr() { this.a = (this.b = 5) + 1; return this.b; }
    static int f() { return 9; }
    void saved() { this.a = this.b = this.c = f(); }
    void holderChain() { h().a = h().b = 5; }
    NEG h() { return this; }
    public static void main(String[] args) {
        NEG n = new NEG();
        n.cross();
        System.out.println("" + o1.a + "/" + o2.b);
        System.out.println("" + n.expr());
        n.saved();
        System.out.println("" + n.a + "/" + n.b + "/" + n.c);
        n.holderChain();
        System.out.println("" + n.a + "/" + n.b);
    }
}
