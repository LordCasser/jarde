public class IntegerSwitchAudit {
    static final int LOW = 0xABC;
    static final int HIGH = 0xCDE;

    public static int grouped(int x) {
        switch (x) {
            case 1:
            case 7:
                return 11;
            case 2:
                return 20;
            default:
                return -1;
        }
    }

    public static int fallthrough(int x) {
        int value = 3;
        switch (x) {
            case 10:
                value += 4;
            case 20:
                value *= 2;
                break;
            case 30:
                value = -5;
                break;
        }
        return value;
    }

    public static int noDefault(int x) {
        int value = 99;
        switch (x) {
            case 4:
                value = LOW;
                break;
            case 8:
                value = HIGH;
                break;
        }
        return value;
    }

    public static int labelConstant(int x) {
        switch (x) {
            case LOW:
                return HIGH;
            default:
                return 0;
        }
    }
}
