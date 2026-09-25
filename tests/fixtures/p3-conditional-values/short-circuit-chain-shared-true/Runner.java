public final class Runner {
    private static void check(boolean extra, boolean left, boolean rhsValue) {
        ChainOrField.result = false;
        ChainOrField.rhsValue = rhsValue;
        ChainOrField.calls = 0;
        ChainOrField.assign(extra, left);
        System.out.println("extra=" + extra + ",left=" + left + ",rhs=" + rhsValue
                + ",result=" + ChainOrField.result + ",calls=" + ChainOrField.calls);
    }

    public static void main(String[] args) {
        check(true, false, false);
        check(false, true, false);
        check(false, false, true);
        check(false, false, false);
    }
}
