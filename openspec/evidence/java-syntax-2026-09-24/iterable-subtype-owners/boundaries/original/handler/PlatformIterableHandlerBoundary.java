import java.util.AbstractCollection;
import java.util.Collection;
import java.util.Iterator;

final class PlatformIterableHandlerBoundary {
    static int caught(Collection values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            try {
                Object item = it.next();
                result += item.toString().length();
            } catch (IllegalStateException failure) {
                result++;
            }
        }
        return result;
    }

    static final class OneShotFailure extends AbstractCollection {
        public Iterator iterator() {
            return new Iterator() {
                int index;

                public boolean hasNext() { return index == 0; }
                public Object next() {
                    index++;
                    throw new IllegalStateException("next");
                }
                public void remove() { throw new UnsupportedOperationException(); }
            };
        }

        public int size() { return 1; }
    }
}
