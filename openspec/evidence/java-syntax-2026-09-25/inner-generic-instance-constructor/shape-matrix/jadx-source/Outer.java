package matrix;

/* JADX INFO: loaded from: original-input-g-none.jar:matrix/Outer.class */
public class Outer {

    /* JADX INFO: loaded from: original-input-g-none.jar:matrix/Outer$A.class */
    public static class A<T> {
        public static int effects;

        /* JADX INFO: loaded from: original-input-g-none.jar:matrix/Outer$A$Generic.class */
        public class Generic<V> {
            private final V value;

            public Generic(V v) {
                this.value = v;
            }

            public V value() {
                return this.value;
            }
        }

        /* JADX INFO: loaded from: original-input-g-none.jar:matrix/Outer$A$Plain.class */
        public class Plain {
            public Plain(int i) {
            }
        }

        public static int mark(int i) {
            effects++;
            return i;
        }
    }
}
