import java.util.Iterator;

public final class CatchNextScopeRunner {
    public static void main(String[] args) {
        final int[] nextCalls = {0};
        Iterable throwsOnceInNext = new Iterable() {
            public Iterator iterator() {
                return new Iterator() {
                    public boolean hasNext() { return nextCalls[0] == 0; }
                    public Object next() {
                        nextCalls[0]++;
                        throw new IllegalStateException("next");
                    }
                    public void remove() { throw new UnsupportedOperationException(); }
                };
            }
        };
        try {
            System.out.println("nextHandler=caught:"
                    + CatchNextScope.consume(throwsOnceInNext));
        } catch (RuntimeException e) {
            System.out.println("nextHandler=escaped:" + e.getClass().getSimpleName());
        }
    }
}
