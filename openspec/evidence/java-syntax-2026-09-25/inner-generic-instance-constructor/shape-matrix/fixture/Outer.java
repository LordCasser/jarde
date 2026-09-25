package matrix;

public class Outer {
    public static class A<T> {
        public static int effects;

        public static int mark(int value) {
            effects++;
            return value;
        }

        public class Plain {
            public Plain(int value) {}
        }

        public class Generic<V> {
            private final V value;

            public Generic(V value) {
                this.value = value;
            }

            public V value() {
                return value;
            }
        }
    }
}
