public final class ReferenceDevDriver {
    private static void check(boolean ok, String label) {
        if (!ok) throw new AssertionError(label);
    }

    public static void main(String[] args) {
        check(ReferenceCases.interfaceCall(true) == 31, "interface first");
        check(ReferenceCases.interfaceCall(false) == 47, "interface second");
        check("north".equals(ReferenceCases.nullCopyChain(true)), "copy north");
        check("south".equals(ReferenceCases.nullCopyChain(false)), "copy south");
        check(ReferenceCases.covariantArrayRead(true) == 'a', "string array");
        check(ReferenceCases.covariantArrayRead(false) == 'g', "builder array");
        Object first = ReferenceCases.objectExchange(true);
        Object second = ReferenceCases.objectExchange(false);
        check(first instanceof Integer && ((Integer) first).intValue() == 73, "object exchange first");
        check("right-value".equals(second), "object exchange second");
        check("cast-ok".equals(ReferenceCases.explicitCast(true)), "cast succeeds");
        boolean failed = false;
        try {
            ReferenceCases.explicitCast(false);
        } catch (ClassCastException expected) {
            failed = true;
        }
        check(failed, "cast failure branch");
        System.out.println("reference-dev-ok");
    }
}
