// The `ok &= x > 0` reproduction probe: the critical anchor 15's own shape, frozen for the
// follow-up slice. javac lowers the comparison to a branch whose arms push the 0/1 the `iand`
// consumes, so the compound's right-hand value is materialized **across** the branch.
public class BE {
    boolean ok = false;
    boolean earlyRet(int[] xs) { for (int x : xs) { ok &= x > 0; if (!ok) { return false; } } return true; }
    public static void main(String[] a) {
        BE b = new BE();
        System.out.println("" + b.earlyRet(new int[]{1, -2, 3}));
    }
}
