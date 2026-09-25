public class AssertVariants {
    static int effects;

    static {
        effects++;
    }

    private static boolean probe(boolean value) {
        effects = effects * 10 + 2;
        return value;
    }

    private static String detail() {
        effects = effects * 10 + 3;
        return "message";
    }

    static int noMessage(boolean condition) {
        assert probe(condition);
        return effects;
    }

    static int multiple(boolean first, boolean second) {
        assert probe(first);
        assert probe(second) : detail();
        return effects;
    }
}
