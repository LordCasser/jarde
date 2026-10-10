package defpackage;

/* JADX INFO: loaded from: NoClinitSuperArgumentTargets.jar:CommonNoClinitArrayInit.class */
public class CommonNoClinitArrayInit extends ArrayFieldInitBase {
    static int trace;
    final byte[] first;
    final byte[] second;

    static byte mark(int i) {
        trace = (trace * 31) + i;
        return (byte) i;
    }

    static byte run(int i) {
        return mark(i);
    }

    public CommonNoClinitArrayInit() {
        super(7);
        this.first = new byte[]{mark(11), mark(12)};
        this.second = new byte[]{run(21)};
        trace = (trace * 31) + 91;
    }

    public CommonNoClinitArrayInit(int i) {
        super(i);
        this.first = new byte[]{mark(11), mark(12)};
        this.second = new byte[]{run(21)};
        trace = (trace * 31) + i;
    }
}
