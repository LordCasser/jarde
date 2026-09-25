package defpackage;

import java.util.Collection;
import java.util.Iterator;
import java.util.List;

/* JADX INFO: loaded from: PlatformIterableOwners.class */
final class PlatformIterableOwners {
    static int touches;

    PlatformIterableOwners() {
    }

    static int listCast(List list) {
        Iterator it = list.iterator();
        int length = 0;
        while (true) {
            int i = length;
            if (!it.hasNext()) {
                return i;
            }
            String str = (String) it.next();
            touches++;
            length = i + str.length();
        }
    }

    static int collectionCast(Collection collection) {
        Iterator it = collection.iterator();
        int length = 0;
        while (true) {
            int i = length;
            if (!it.hasNext()) {
                return i;
            }
            String str = (String) it.next();
            touches++;
            length = i + str.length();
        }
    }

    static int listSkipEmpty(List list) {
        Iterator it = list.iterator();
        int length = 0;
        while (it.hasNext()) {
            String str = (String) it.next();
            if (!str.isEmpty()) {
                length += str.length();
            }
        }
        return length;
    }

    static int listConsumesTwice(List list) {
        Iterator it = list.iterator();
        int length = 0;
        while (true) {
            int i = length;
            if (!it.hasNext()) {
                return i;
            }
            length = i + it.next().toString().length() + it.next().toString().length();
        }
    }

    static int collectionEscapes(Collection collection) {
        int i;
        Iterator it = collection.iterator();
        int length = 0;
        while (true) {
            i = length;
            if (!it.hasNext()) {
                break;
            }
            length = i + ((String) it.next()).length();
        }
        return i + (it.hasNext() ? 1 : 0);
    }

    static int userSubtype(TextIterable textIterable) {
        Iterator it = textIterable.iterator();
        int length = 0;
        while (true) {
            int i = length;
            if (!it.hasNext()) {
                return i;
            }
            length = i + it.next().toString().length();
        }
    }

    static int sameNameOnly(IteratorSurface iteratorSurface) {
        Iterator it = iteratorSurface.iterator();
        int length = 0;
        while (true) {
            int i = length;
            if (!it.hasNext()) {
                return i;
            }
            length = i + it.next().toString().length();
        }
    }
}
