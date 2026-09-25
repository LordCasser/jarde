public final class InitEffects {
    static String trace = "";
    static int count;

    private InitEffects() {}

    static String next(String label) {
        trace += label;
        return label + ++count;
    }

    static int total(String first, String second) {
        trace += "T";
        return first.length() + second.length();
    }
}
