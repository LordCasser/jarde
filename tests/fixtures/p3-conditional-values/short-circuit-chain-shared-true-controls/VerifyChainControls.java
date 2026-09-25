public final class VerifyChainControls {
    private static void run(boolean extra, boolean left, boolean rhs) {
        ChainOrFieldDuplicatePhi.rhsValue = rhs;
        ChainOrFieldDuplicatePhi.calls = 0;
        ChainOrFieldDuplicatePhi.assign(extra, left);
        System.out.println(
            "extra=" + extra + ",left=" + left + ",rhs=" + rhs
                + ",result=" + ChainOrFieldDuplicatePhi.result
                + ",mirror=" + ChainOrFieldDuplicatePhi.mirror
                + ",calls=" + ChainOrFieldDuplicatePhi.calls);
    }

    public static void main(String[] args) {
        run(true, false, false);
        run(false, true, false);
        run(false, false, true);
        run(false, false, false);
    }
}
