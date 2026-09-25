
/* JADX INFO: loaded from: JadxOrderingControl.class */
public final class JadxOrderingControl {
    static int trace;

    static int mark(int i) {
        trace = (trace * 10) + i;
        return i;
    }

    static int[] build() {
        return new int[]{mark(2), mark(1)};
    }
}
