package defpackage;

/* JADX INFO: loaded from: Order.class */
public class Order {
    static int trace;

    static boolean[] array(boolean[] zArr) {
        trace = (trace * 10) + 1;
        return zArr;
    }

    static int index() {
        trace = (trace * 10) + 2;
        return 0;
    }

    static int value(int i) {
        trace = (trace * 10) + 3;
        if (i == 99) {
            throw new IllegalStateException("value producer");
        }
        return i;
    }

    /* JADX WARN: Multi-variable type inference failed */
    /* JADX WARN: Type inference failed for: r2v1, types: [int] */
    public static boolean put(boolean[] zArr, int i) {
        array(zArr)[index()] = value(i);
        return zArr[0];
    }
}
