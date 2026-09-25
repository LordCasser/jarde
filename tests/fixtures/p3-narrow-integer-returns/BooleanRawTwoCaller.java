public final class BooleanRawTwoCaller {
    private BooleanRawTwoCaller() {}

    public static int byteValue() {
        return BooleanReturnBoundaries.booleanAsByte(false);
    }

    public static int charValue() {
        return BooleanReturnBoundaries.booleanAsChar(false);
    }

    public static int shortValue() {
        return BooleanReturnBoundaries.booleanAsShort(false);
    }
}
