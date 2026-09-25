package nested;

/* JADX INFO: loaded from: wrong-relation.jar:nested/SimpleOuter.class */
public final class SimpleOuter {
    public static String trace = "";
    private final int bias;

    /* JADX INFO: loaded from: wrong-relation.jar:nested/SimpleOuter$Inner.class */
    public final class Inner {
        private final int value;

        public Inner(int i) {
            this.value = i;
        }

        public int value() {
            return SimpleOuter.this.bias + this.value;
        }
    }

    public SimpleOuter(int i) {
        this.bias = i;
    }

    public static int mark(String str, int i) {
        trace += str;
        return i;
    }
}
