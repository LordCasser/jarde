public class CP {
    int a, b, c;
    void inst() { this.a = this.b = this.c = 5; }
    void pair() { this.a = this.b = 7; }
    public static void main(String[] args) {
        CP p = new CP();
        p.inst();
        System.out.println("" + p.a + "/" + p.b + "/" + p.c);
        p.pair();
        System.out.println("" + p.a + "/" + p.b + "/" + p.c);
    }
}
