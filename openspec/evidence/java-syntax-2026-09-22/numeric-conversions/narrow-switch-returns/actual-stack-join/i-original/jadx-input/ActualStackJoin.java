package defpackage;

/* JADX INFO: loaded from: ActualStackJoin.class */
public final class ActualStackJoin {
    private ActualStackJoin() {
    }

    public static int runByte(int i) {
        int i2;
        switch (i) {
            case 1:
                i2 = 130;
                break;
            default:
                i2 = -129;
                break;
        }
        return i2;
    }

    public static int runChar(int i) {
        int i2;
        switch (i) {
            case 1:
                i2 = 65535;
                break;
            default:
                i2 = -2;
                break;
        }
        return i2;
    }

    public static int runShort(int i) {
        int i2;
        switch (i) {
            case 1:
                i2 = 32768;
                break;
            default:
                i2 = -32769;
                break;
        }
        return i2;
    }
}
