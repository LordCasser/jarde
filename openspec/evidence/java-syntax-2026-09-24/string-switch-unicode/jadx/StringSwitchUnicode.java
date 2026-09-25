package defpackage;

/* JADX INFO: loaded from: StringSwitchUnicode.class */
public final class StringSwitchUnicode {
    private static int calls;

    private static String selector(String str) {
        calls++;
        return str;
    }

    public static String choose(String str) {
        switch (selector(str)) {
            case "":
                return "empty";
            case "雪":
                return "bmp";
            case "𐐷":
                return "supplementary";
            default:
                return "default";
        }
    }

    public static void main(String[] strArr) {
        run("empty", "");
        run("bmp", "雪");
        run("supplementary", "𐐷");
        run("default", "other");
        calls = 0;
        try {
            choose(null);
            System.out.println("null:returned:calls=" + calls);
        } catch (RuntimeException e) {
            System.out.println("null:" + e.getClass().getSimpleName() + ":calls=" + calls);
        }
    }

    private static void run(String str, String str2) {
        calls = 0;
        System.out.println(str + ":" + choose(str2) + ":calls=" + calls);
    }
}
