public final class CompoundConditionNegative {
    private CompoundConditionNegative() {}

    public static String prefixIncrement(boolean a, int initial, boolean c, boolean d) {
        int x = initial;
        StringBuilder payload = new StringBuilder();
        if (a && (++x > 0) || c && d) {
            payload.append("hit");
        } else {
            payload.append("miss");
        }
        return payload.toString() + ":" + x;
    }

    public static String localAssignment(boolean a, int initial, boolean c, boolean d) {
        int x = initial;
        StringBuilder payload = new StringBuilder();
        if (a && ((x = x + 1) > 0) || c && d) {
            payload.append("hit");
        } else {
            payload.append("miss");
        }
        return payload.toString() + ":" + x;
    }
}
