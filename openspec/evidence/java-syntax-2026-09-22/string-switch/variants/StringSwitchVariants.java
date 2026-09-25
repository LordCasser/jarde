public class StringSwitchVariants {
    static int calls;

    static String read(String value) {
        calls++;
        if ("!".equals(value)) {
            throw new IllegalStateException("selector");
        }
        return value;
    }

    public static int choose(String value) {
        switch (value) {
            case "Aa":
            case "BB":
                return 11;
            case "x":
                calls += 10;
            case "y":
                return calls + 20;
            case "":
                return 1;
            case "雪":
                return 9;
            default:
                return 44;
        }
    }

    public static int once(String value) {
        return choose(read(value));
    }
}
