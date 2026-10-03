public class IV2 {
    private int count = 10;
    class Inner {
        void bump() { IV2.this.count = IV2.this.count + 1; }
    }
    int get() { return count; }
    public static void main(String[] args) {
        IV2 o = new IV2();
        o.new Inner().bump();
        System.out.println(o.get());
    }
}
