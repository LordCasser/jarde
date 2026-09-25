package negative;

/* JADX INFO: loaded from: NegativeOuter.class */
public final class NegativeOuter {
    public static String trace = "";

    /* JADX INFO: loaded from: NegativeOuter$Inner.class */
    public final class Inner {
        public final int value;

        public Inner(int i) {
            this.value = i;
        }
    }

    public static int mark(String str, int i) {
        trace += str;
        return i;
    }
}
