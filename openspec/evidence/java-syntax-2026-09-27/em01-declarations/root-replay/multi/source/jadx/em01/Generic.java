package em01;

/* JADX INFO: loaded from: fixture.jar:em01/Generic.class */
public class Generic {

    /* JADX INFO: loaded from: fixture.jar:em01/Generic$A.class */
    public static abstract class A<T> implements Comparable<A<T>> {
        T value;

        @Override // java.lang.Comparable
        public int compareTo(A<T> a) {
            return 0;
        }
    }
}
