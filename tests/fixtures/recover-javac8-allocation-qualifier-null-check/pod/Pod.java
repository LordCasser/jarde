public class Pod {
    private int seed = 6;
    class Nut {
        private int mark;
        Nut() { this.mark = 21; }
        int mark() { return mark + seed; }
    }
    public static void main(String[] args) {
        System.out.println(new Pod().new Nut().mark());
    }
}
