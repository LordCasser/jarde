package minimal;

public class Outer {
    public class Inner<V> {
        private final V value;

        public Inner(V value) {
            this.value = value;
        }

        public V value() {
            return value;
        }
    }
}
