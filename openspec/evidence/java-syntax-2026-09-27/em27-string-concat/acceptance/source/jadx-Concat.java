package em27;

/* JADX INFO: loaded from: input.jar:em27/Concat.class */
public final class Concat {
    private Concat() {
    }

    public static String builder(int i) {
        return "Value equals " + i;
    }

    public static String folded() {
        return "App version: 1.2";
    }

    public static String objects(Object obj, Object obj2) {
        StringBuilder sb = new StringBuilder();
        sb.append(obj);
        sb.append('=');
        sb.append(obj2);
        return sb.toString();
    }

    public static String character(String str) {
        return '1' + str + ", e: 2";
    }

    public static void discarded(int i) {
        String str = "Input arg value: " + i;
    }

    public static String explicitConstructor() {
        return "abc";
    }

    public static String explicitStored() {
        return "abc";
    }
}
