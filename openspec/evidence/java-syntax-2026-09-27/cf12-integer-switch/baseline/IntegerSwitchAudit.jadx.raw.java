package defpackage;

/* JADX INFO: loaded from: input.jar:IntegerSwitchAudit.class */
public class IntegerSwitchAudit {
    static final int LOW = 2748;
    static final int HIGH = 3294;

    public static int grouped(int i) {
        switch (i) {
            case 1:
            case 7:
                return 11;
            case 2:
                return 20;
            default:
                return -1;
        }
    }

    public static int fallthrough(int i) {
        int i2 = 3;
        switch (i) {
            case 10:
                i2 = 3 + 4;
            case 20:
                i2 *= 2;
                break;
            case 30:
                i2 = -5;
                break;
        }
        return i2;
    }

    public static int noDefault(int i) {
        int i2 = 99;
        switch (i) {
            case 4:
                i2 = LOW;
                break;
            case 8:
                i2 = HIGH;
                break;
        }
        return i2;
    }

    public static int labelConstant(int i) {
        switch (i) {
            case LOW /* 2748 */:
                return HIGH;
            default:
                return 0;
        }
    }
}
