public final class StringSwitchUnicode {
    private static int calls;

    private static String selector(String value) {
        calls++;
        return value;
    }

    public static String choose(String value) {
        switch (selector(value)) {
            case "": return "empty";
            case "雪": return "bmp";
            case "𐐷": return "supplementary";
            default: return "default";
        }
    }

    public static void main(String[] args) {
        run("empty", "");
        run("bmp", "雪");
        run("supplementary", "𐐷");
        run("default", "other");
        calls = 0;
        try {
            choose(null);
            System.out.println("null:returned:calls=" + calls);
        } catch (RuntimeException ex) {
            System.out.println("null:" + ex.getClass().getSimpleName() + ":calls=" + calls);
        }
    }

    private static void run(String label, String value) {
        calls = 0;
        System.out.println(label + ":" + choose(value) + ":calls=" + calls);
    }
}
