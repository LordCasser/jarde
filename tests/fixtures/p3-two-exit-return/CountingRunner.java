public final class CountingRunner {
    private static CountedValue value(String text) { return text == null ? null : new CountedValue(text); }
    private static void check(String la, String lb, String ra, String rb) {
        CountedValue.calls = 0;
        CountingProbe left = new CountingProbe(value(la), value(lb));
        CountingProbe right = new CountingProbe(value(ra), value(rb));
        System.out.println(left.bothMatch(right) + ":" + CountedValue.calls);
    }
    public static void main(String[] args) {
        check(null, null, null, null);
        check("a", "b", "a", "b");
        check(null, "b", "a", "b");
        check("a", "b", null, "b");
        check("a", "b", "x", "b");
        check("a", null, "a", "b");
        check("a", "b", "a", null);
        check("a", "b", "a", "x");
    }
}
