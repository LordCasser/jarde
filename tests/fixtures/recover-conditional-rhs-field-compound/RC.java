public class RC {
    boolean ok = false;
    int n = 3;

    // The critical-15 shape: the instance boolean compound inside a surviving enhanced `for`, the
    // comparison materialized across the branch's two arms and the join.
    boolean earlyRet(int[] xs) {
        for (int x : xs) {
            ok &= x > 0;
            if (!ok) { return false; }
        }
        return true;
    }

    // The same materialization with no loop around it: the statement's own block ends in the branch.
    void plain(int x) { ok &= x > 0; }

    // The same shape with the other eager boolean operator.
    void orEq(int x) { ok |= x > 0; }

    // The integral field: the same materialization, with the sibling operand an `int` — the arms
    // keep the `? 1 : 0` spelling the position states.
    void mask(int x) { n &= (x > 0) ? 1 : 0; }

    public static void main(String[] a) {
        RC early = new RC();
        early.ok = true;
        System.out.println(early.earlyRet(new int[] {1, -2, 3}));
        RC plain = new RC();
        plain.plain(-1);
        System.out.println(plain.ok);
        RC or = new RC();
        or.orEq(1);
        System.out.println(or.ok);
        RC mask = new RC();
        mask.mask(1);
        System.out.println(mask.n);
    }
}
