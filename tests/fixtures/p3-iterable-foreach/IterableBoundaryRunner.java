import java.util.Arrays;
import java.util.Iterator;

final class IterableBoundaryRunner {
    public static void main(String[] args) {
        IterableBoundary.calls = 0;
        IterableBoundary.supplied = Arrays.asList("a", "bc");
        System.out.println("captured=" + IterableBoundary.capturedOnce()
                + ",calls=" + IterableBoundary.calls);
        System.out.println("skip=" + IterableBoundary.skipEmpty(Arrays.asList("", "ab")));
        IterableBoundary.calls = 0;
        IterableBoundary.supplied = Arrays.asList();
        System.out.println("empty=" + IterableBoundary.capturedOnce()
                + ",calls=" + IterableBoundary.calls);
        IterableBoundary.calls = 0;
        IterableBoundary.supplied = null;
        try {
            IterableBoundary.capturedOnce();
        } catch (RuntimeException e) {
            System.out.println("null=" + e.getClass().getSimpleName()
                    + ",calls=" + IterableBoundary.calls);
        }
        for (int stage = 0; stage < 3; stage++) {
            final int failingStage = stage;
            IterableBoundary.calls = 0;
            IterableBoundary.supplied = new Iterable() {
                public Iterator iterator() {
                    if (failingStage == 0) throw new IllegalStateException("iterator");
                    return new Iterator() {
                        public boolean hasNext() {
                            if (failingStage == 1) throw new IllegalStateException("hasNext");
                            return true;
                        }
                        public Object next() {
                            throw new IllegalStateException("next");
                        }
                        public void remove() { throw new UnsupportedOperationException(); }
                    };
                }
            };
            try {
                IterableBoundary.capturedOnce();
            } catch (RuntimeException e) {
                System.out.println("throw" + stage + "=" + e.getClass().getSimpleName()
                        + ",calls=" + IterableBoundary.calls);
            }
        }
    }
}
