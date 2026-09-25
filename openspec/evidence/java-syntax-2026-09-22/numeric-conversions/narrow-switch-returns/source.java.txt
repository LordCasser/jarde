public final class NarrowSwitchReturns {
    private NarrowSwitchReturns() {}

    public static int runByte(int tag) {
        int value;
        switch (tag) {
            case 1:
                value = 130;
                break;
            default:
                value = -129;
                break;
        }
        return value;
    }

    public static int runChar(int tag) {
        int value;
        switch (tag) {
            case 1:
                value = 65535;
                break;
            default:
                value = -2;
                break;
        }
        return value;
    }

    public static int runShort(int tag) {
        int value;
        switch (tag) {
            case 1:
                value = 32768;
                break;
            default:
                value = -32769;
                break;
        }
        return value;
    }

    public static int boolByte(boolean flag) {
        int value;
        if (flag) {
            value = 255;
        } else {
            value = -256;
        }
        return value;
    }

    public static int boolChar(boolean flag) {
        int value;
        if (flag) {
            value = 65535;
        } else {
            value = -2;
        }
        return value;
    }

    public static int boolShort(boolean flag) {
        int value;
        if (flag) {
            value = 32768;
        } else {
            value = -32769;
        }
        return value;
    }
}
