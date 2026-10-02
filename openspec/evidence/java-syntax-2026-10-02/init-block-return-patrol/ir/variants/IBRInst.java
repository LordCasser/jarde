public class IBRInst {
    int x;
    int y;
    {
        x = 4;
        if (x < 0) { throw new IllegalStateException("never"); }
    }
    {
        y = x * 2;
    }
    IBRInst() {
    }
    IBRInst(int seed) {
        y = y + seed;
    }
    public static void main(String[] args) {
        IBRInst one = new IBRInst();
        IBRInst two = new IBRInst(10);
        System.out.println(one.x + ":" + one.y + ":" + two.x + ":" + two.y);
    }
}
