public final class NullPhiChoicesDriver {
    public static void main(String[] args) {
        if (NullPhiChoices.bothNull(true) != null) throw new AssertionError("bothNull(true)");
        if (NullPhiChoices.bothNull(false) != null) throw new AssertionError("bothNull(false)");
        CharSequence left = NullPhiChoices.differingReferences(true);
        if (!(left instanceof String) || !"left".contentEquals(left)) throw new AssertionError("differingReferences(true)");
        CharSequence right = NullPhiChoices.differingReferences(false);
        if (!(right instanceof StringBuilder) || !"right".contentEquals(right)) throw new AssertionError("differingReferences(false)");
        System.out.println("PASS: bothNull={true,false}; differingReferences={String:left,StringBuilder:right}");
    }
}
