package defpackage;

/* JADX INFO: loaded from: ArrayFieldInitializers.class */
public class ArrayFieldInitializers {
    static int trace = 0;
    static byte before = mark(1);
    static byte[] a = {mark(2), mark(3), mark(4)};
    static byte after = mark(5);
    byte[] b = {mark(6), mark(7), mark(8)};

    public ArrayFieldInitializers() {
        mark(9);
    }

    static byte mark(int i) {
        trace = (trace * 10) + i;
        return (byte) i;
    }
}
