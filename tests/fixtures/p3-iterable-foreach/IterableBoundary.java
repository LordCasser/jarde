import java.util.Collection;
import java.util.Iterator;
import java.util.List;

final class IterableBoundary {
    static int calls;
    static Iterable supplied;

    static Iterable supply() {
        calls++;
        return supplied;
    }

    static int capturedOnce() {
        Iterable values = supply();
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            Object item = it.next();
            String text = (String) item;
            result += text.length();
        }
        return result;
    }

    static int skipEmpty(Iterable values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            Object item = it.next();
            String text = (String) item;
            if (text.isEmpty()) {
                continue;
            }
            result += text.length();
        }
        return result;
    }

    static int listOwner(List<String> values) {
        Iterator<String> it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            String value = it.next();
            result += value.length();
        }
        return result;
    }

    static int collectionOwner(Collection<String> values) {
        Iterator<String> it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            String value = it.next();
            result += value.length();
        }
        return result;
    }

    static int twoNext(Iterable values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            Object first = it.next();
            Object second = it.next();
            result += first.toString().length() + second.toString().length();
        }
        return result;
    }

    static int iteratorEscapes(Iterable values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            Object item = it.next();
            String text = (String) item;
            result += text.length();
        }
        return result + (it.hasNext() ? 1 : 0);
    }
}
