public class MX {
    int a, b;
    static int s;
    void mixStaticRight() { this.a = this.b = MX.s = 5; }
    void mixStaticLeft() { MX.s = this.a = this.b = 6; }
    void mixStaticMid() { this.a = MX.s = this.b = 7; }
    public static void main(String[] args) {
        MX m = new MX();
        m.mixStaticRight(); System.out.println(m.a + "/" + m.b + "/" + MX.s);
        m.mixStaticLeft();  System.out.println(m.a + "/" + m.b + "/" + MX.s);
        m.mixStaticMid();   System.out.println(m.a + "/" + m.b + "/" + MX.s);
    }
}
