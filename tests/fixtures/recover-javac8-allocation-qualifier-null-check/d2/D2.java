public class D2 {
    private int f = 3;
    class In {
        int v() { return f + 1; }
    }
    int hit() { return 1; }
    int drive() {
        D2 o = new D2();
        o.hit();
        return o.new In().v();
    }
    public static void main(String[] args) {
        System.out.println(new D2().drive());
    }
}
