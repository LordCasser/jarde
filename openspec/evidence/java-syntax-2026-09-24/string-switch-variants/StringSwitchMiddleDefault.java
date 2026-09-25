public class StringSwitchMiddleDefault {
    static int calls;
    static int score;

    static String read(String value) {
        calls++;
        if ("!".equals(value)) {
            throw new IllegalStateException("selector");
        }
        return value;
    }

    static void add(int value) {
        score = score * 100 + value;
    }

    public static int choose(String value) {
        score = 0;
        switch (read(value)) {
            case "Aa":
            case "BB":
                add(1);
                break;
            default:
                add(2);
            case "":
                add(3);
                break;
            case "雪":
                add(4);
                break;
            case "prefix":
                add(5);
            case "suffix":
                add(6);
                break;
        }
        return score;
    }

    public static int once(String value) {
        return choose(value);
    }
}
