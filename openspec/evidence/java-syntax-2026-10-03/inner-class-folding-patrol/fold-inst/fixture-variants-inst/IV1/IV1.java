public class IV1 {
    private int base = 4;
    private int bonus = 3;
    class Inner {
        private int tag;
        Inner(int t) { this.tag = t; }
        int total() { return tag + base + bonus; }
    }
    Inner make(int t) { return new Inner(t); }
    public static void main(String[] args) {
        System.out.println(new IV1().make(5).total());
    }
}
