public final class NullPhiChoices {
    public static Object bothNull(boolean flag) {
        Object value = flag ? null : null;
        return value;
    }

    public static CharSequence differingReferences(boolean flag) {
        CharSequence value = flag ? "left" : new StringBuilder("right");
        return value;
    }
}
