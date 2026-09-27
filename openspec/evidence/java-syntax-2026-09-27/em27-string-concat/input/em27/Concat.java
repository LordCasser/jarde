package em27;

public final class Concat {
    private Concat() {}

    public static String builder(int value) {
        return new StringBuilder().append("Value").append(" equals ").append(value).toString();
    }

    public static String folded() {
        return new StringBuilder().append("App ").append("version: ").append(1).append('.')
                .append(2).toString();
    }

    public static String objects(Object first, Object second) {
        StringBuilder builder = new StringBuilder();
        builder.append(first);
        builder.append('=');
        builder.append(second);
        return builder.toString();
    }

    public static String character(String name) {
        return '1' + name + ", e: " + 2;
    }

    public static void discarded(int value) {
        String message = "Input arg value: " + value;
        if (false) System.out.println(message);
    }

    public static String explicitConstructor() {
        return new String(new char[] {'a', 'b', 'c'});
    }

    public static String explicitStored() {
        char[] characters = new char[] {'a', 'b', 'c'};
        return new String(characters);
    }
}
