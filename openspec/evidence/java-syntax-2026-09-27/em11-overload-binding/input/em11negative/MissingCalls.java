package em11negative;

public class MissingCalls {
    public static String take(MissingBase value) { return "base"; }
    public static String take(MissingChild value) { return "child"; }

    public static String run() {
        return take((MissingBase) new MissingChild());
    }
}
