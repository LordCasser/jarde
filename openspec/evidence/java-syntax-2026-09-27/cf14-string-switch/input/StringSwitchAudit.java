public class StringSwitchAudit {
    static int calls;

    static String read(String value) {
        calls++;
        return value;
    }

    public static int choose(String value) {
        switch (read(value)) {
            case "frewhyh":
                return 1;
            case "phgafkp":
                return 2;
            case "test":
            case "test2":
                return 3;
            case "other":
                return 4;
            default:
                return 0;
        }
    }
}
