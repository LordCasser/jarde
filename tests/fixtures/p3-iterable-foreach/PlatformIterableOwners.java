import java.util.Collection;
import java.util.Iterator;
import java.util.List;

final class PlatformIterableOwners {
    static int touches;

    static int listCast(List values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            Object item = it.next();
            String text = (String) item;
            touches++;
            result += text.length();
        }
        return result;
    }

    static int collectionCast(Collection values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            Object item = it.next();
            String text = (String) item;
            touches++;
            result += text.length();
        }
        return result;
    }

    static int listSkipEmpty(List values) {
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

    static int listConsumesTwice(List values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            Object first = it.next();
            Object second = it.next();
            result += first.toString().length() + second.toString().length();
        }
        return result;
    }

    static int collectionEscapes(Collection values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            Object item = it.next();
            String text = (String) item;
            result += text.length();
        }
        return result + (it.hasNext() ? 1 : 0);
    }

    static int userSubtype(TextIterable values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            Object item = it.next();
            result += item.toString().length();
        }
        return result;
    }

    static int sameNameOnly(IteratorSurface values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            Object item = it.next();
            result += item.toString().length();
        }
        return result;
    }
}
