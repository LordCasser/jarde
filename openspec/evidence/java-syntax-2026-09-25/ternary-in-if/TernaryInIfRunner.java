public final class TernaryInIfRunner {
    private static void check(String label, String leftA, String leftB, String rightA, String rightB,
            boolean expected) {
        TernaryInIfProbe left = new TernaryInIfProbe(leftA, leftB);
        TernaryInIfProbe right = new TernaryInIfProbe(rightA, rightB);
        boolean actual = left.bothMatch(right);
        System.out.println(label + ":" + actual + ":" + expected);
    }

    public static void main(String[] args) {
        check("both-null", null, null, null, null, true);
        check("both-non-null-equal", "a", "b", "a", "b", true);
        check("a-left-null", null, "b", "a", "b", false);
        check("a-right-null", "a", "b", null, "b", false);
        check("a-different", "a", "b", "x", "b", false);
        check("b-left-null", "a", null, "a", "b", false);
        check("b-right-null", "a", "b", "a", null, false);
        check("b-different", "a", "b", "a", "x", false);
    }
}
