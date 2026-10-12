package cf11;

public final class CF11CharSequenceControl {
    public static int stringLength() {
        CharSequence value = "string";
        return value.length();
    }

    public static int builderLength() {
        CharSequence value = "seed";
        value = new StringBuilder("builder");
        return value.length();
    }
}
