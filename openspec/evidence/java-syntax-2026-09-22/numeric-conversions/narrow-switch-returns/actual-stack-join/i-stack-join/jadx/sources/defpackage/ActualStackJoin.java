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

    public static int runShort(int i) {
        switch (i) {
            case 1:
                return 32768;
            default:
                return -32769;
        }
    }
}
