package minimal;

/* JADX INFO: loaded from: Outer.class */
public class Outer {

    /* JADX INFO: loaded from: Outer$Inner.class */
    public class Inner<V> {
        private final V value;

        public Inner(V value) {
            this.value = value;
        }

        public V value() {
            return this.value;
        }
    }
}
