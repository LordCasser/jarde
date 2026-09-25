package defpackage;

import java.util.Collection;
import java.util.Iterator;
import java.util.List;

/* JADX INFO: loaded from: PlatformIterableOwners.class */
final class PlatformIterableOwners {
    static int touches;

    PlatformIterableOwners() {
    }

    static int listCast(List values) {
        Iterator it = values.iterator();
        int length = 0;
        while (true) {
            int result = length;
            if (it.hasNext()) {
                Object item = it.next();
                String text = (String) item;
                touches++;
                length = result + text.length();
            } else {
                return result;
            }
        }
    }

    static int collectionCast(Collection values) {
        Iterator it = values.iterator();
        int length = 0;
        while (true) {
            int result = length;
            if (it.hasNext()) {
                Object item = it.next();
                String text = (String) item;
                touches++;
                length = result + text.length();
            } else {
                return result;
            }
        }
    }

    static int listSkipEmpty(List values) {
        int result = 0;
        for (Object item : values) {
            String text = (String) item;
            if (!text.isEmpty()) {
                result += text.length();
            }
        }
        return result;
    }

    static int listConsumesTwice(List values) {
        Iterator it = values.iterator();
        int length = 0;
        while (true) {
            int result = length;
            if (it.hasNext()) {
                Object first = it.next();
                Object second = it.next();
                length = result + first.toString().length() + second.toString().length();
            } else {
                return result;
            }
        }
    }

    static int collectionEscapes(Collection values) {
        int result;
        Iterator it = values.iterator();
        int length = 0;
        while (true) {
            result = length;
            if (!it.hasNext()) {
                break;
            }
            Object item = it.next();
            String text = (String) item;
            length = result + text.length();
        }
        return result + (it.hasNext() ? 1 : 0);
    }

    static int userSubtype(TextIterable values) {
        Iterator it = values.iterator();
        int length = 0;
        while (true) {
            int result = length;
            if (it.hasNext()) {
                Object item = it.next();
                length = result + item.toString().length();
            } else {
                return result;
            }
        }
    }

    static int sameNameOnly(IteratorSurface values) {
        Iterator it = values.iterator();
        int length = 0;
        while (true) {
            int result = length;
            if (it.hasNext()) {
                Object item = it.next();
                length = result + item.toString().length();
            } else {
                return result;
            }
        }
    }
}
