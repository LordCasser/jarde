public final class ActualStackJoin {
    private ActualStackJoin() {}

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
}
