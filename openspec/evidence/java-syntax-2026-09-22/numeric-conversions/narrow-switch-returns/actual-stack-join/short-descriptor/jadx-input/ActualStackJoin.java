package defpackage;

/* JADX INFO: loaded from: ActualStackJoin.class */
public final class ActualStackJoin {
    private ActualStackJoin() {
    }

    public static int runByte(int i) {
        switch (i) {
            case 1:
                return 130;
            default:
                return -129;
        }
    }

    public static int runChar(int i) {
        switch (i) {
            case 1:
                return 65535;
            default:
                return -2;
        }
    }

    public static short runShort(int i) {
        switch (i) {
            case 1:
                return Short.MIN_VALUE;
            default:
                return Short.MAX_VALUE;
        }
    }
}
