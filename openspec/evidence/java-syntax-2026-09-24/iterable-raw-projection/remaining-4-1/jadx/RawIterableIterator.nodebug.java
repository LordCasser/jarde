package defpackage;

import java.util.Iterator;

/* JADX INFO: loaded from: RawIterableIterator.class */
public final class RawIterableIterator {
    public static int touches;

    public static int rawCast(Iterable iterable) {
        Iterator it = iterable.iterator();
        int length = 0;
        while (true) {
            int i = length;
            if (!it.hasNext()) {
                return i;
            }
            length = i + ((String) it.next()).length();
        }
    }

    public static int nextTwiceCastBeforeTouch(Iterable iterable) {
        Iterator it = iterable.iterator();
        int length = 0;
        while (true) {
            int i = length;
            if (!it.hasNext()) {
                return i;
            }
            Object next = it.next();
            touches++;
            length = i + ((String) next).length() + next.toString().length();
        }
    }

    public static int nextTwiceTouchBeforeCast(Iterable iterable) {
        Iterator it = iterable.iterator();
        int length = 0;
        while (true) {
            int i = length;
            if (!it.hasNext()) {
                return i;
            }
            Object next = it.next();
            touches++;
            length = i + ((String) next).length() + next.toString().length();
        }
    }

    public static int touchBeforeNext(Iterable iterable) {
        Iterator it = iterable.iterator();
        int length = 0;
        while (true) {
            int i = length;
            if (!it.hasNext()) {
                return i;
            }
            touches++;
            length = i + ((String) it.next()).length();
        }
    }
}
