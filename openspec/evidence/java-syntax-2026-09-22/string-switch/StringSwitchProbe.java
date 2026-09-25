public class StringSwitchProbe {
    static int calls;

    static String read(String value) {
        calls++;
        return value;
    }

    public static int choose(String value) {
        switch (value) {
            case "Aa":
                return 11;
            case "BB":
                return 22;
            case "z":
                return 33;
            default:
                return 44;
        }
    }

    public static int once(String value) {
        return choose(read(value));
    }
}
