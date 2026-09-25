package nested;

public class OuterGeneric {
    public static class A<T> {
        public class B<V> {
            private final V value;

            public B(V value) {
                this.value = value;
            }

            public V get() {
                return value;
            }
        }
    }

    public static String run() {
        A<String> a = new A<>();
        A<String>.B<Integer> b = a.new B<Integer>(3);
        return b.get().toString();
    }

    public static void main(String[] args) {
        System.out.println(run());
    }
}
