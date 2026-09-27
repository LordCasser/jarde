package cf06;

public final class NegativeAssignments {
    private static int calls;

    private static int effect() {
        return ++calls;
    }

    public static int interleaved(String text) {
        int length;
        if ((length = text.length()) + effect() > 5) {
            return -1;
        }
        return length;
    }

    public static int exceptional(String text) {
        int length;
        try {
            if ((length = text.length()) > 5) {
                return -1;
            }
        } catch (RuntimeException exception) {
            return -2;
        }
        return length;
    }

    public static int loopCondition(String text) {
        int length;
        while ((length = text.length()) > 5) {
            text = text.substring(1);
        }
        return length;
    }
}
