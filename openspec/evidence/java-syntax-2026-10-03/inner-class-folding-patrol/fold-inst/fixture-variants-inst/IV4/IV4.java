public class IV4 {
    private int base = 4;
    static int access$000(IV4 x) { return x.base + 100; }
    class Inner {
        int total() { return IV4.access$000(IV4.this); }
    }
    public static void main(String[] args) {
        System.out.println(new IV4().new Inner().total());
    }
}
