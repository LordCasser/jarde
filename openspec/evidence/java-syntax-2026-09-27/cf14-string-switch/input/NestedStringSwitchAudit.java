public class NestedStringSwitchAudit {
    public static int choose(String value) {
        switch (value) {
            case "a":
                return 1;
            default:
                switch (value) {
                    case "b":
                        return 2;
                    case "c":
                        return 3;
                    default:
                        return 4;
                }
        }
    }
}
