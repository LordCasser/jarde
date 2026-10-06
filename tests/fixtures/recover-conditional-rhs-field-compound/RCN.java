public class RCN {
    boolean ok = false;
    static int calls = 0;

    static boolean side() { calls++; return true; }

    // The true arm calls: the materialization is not two constants.
    void armCall(boolean b) { ok &= (b ? side() : false); }

    // Two materializations in one right-hand side: the first join is the second branch's block.
    void nested(int x, int y) { ok &= (x > 0) & (y > 0); }

    // The materialization is covered by an exception table.
    void inTry(int x) {
        try {
            ok &= x > 0;
        } catch (RuntimeException e) {
            ok = false;
        }
    }

    public static void main(String[] a) {
        RCN r = new RCN();
        r.armCall(true);
        r.nested(1, 2);
        r.inTry(1);
        System.out.println("" + r.ok + "/" + calls);
    }
}
