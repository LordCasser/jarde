package em01;
public class Generic {
    public abstract static class A<T> implements Comparable<A<T>> {
        T value;
        public int compareTo(A<T> other) { return 0; }
    }
}
