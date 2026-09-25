package defpackage;

import java.util.Iterator;

/* JADX INFO: loaded from: RawIterableIterator.class */
public final class RawIterableIterator {
    public static int touches;

    public static int rawCast(Iterable values) {
        Iterator it = values.iterator();
        int length = 0;
        while (true) {
            int result = length;
            if (it.hasNext()) {
                Object item = it.next();
                String text = (String) item;
                length = result + text.length();
            } else {
                return result;
            }
        }
    }

    public static int nextTwiceCastBeforeTouch(Iterable values) {
        Iterator it = values.iterator();
        int length = 0;
        while (true) {
            int result = length;
            if (it.hasNext()) {
                Object item = it.next();
                String text = (String) item;
                touches++;
                length = result + text.length() + item.toString().length();
            } else {
                return result;
            }
        }
    }

    public static int nextTwiceTouchBeforeCast(Iterable values) {
        Iterator it = values.iterator();
        int length = 0;
        while (true) {
            int result = length;
            if (it.hasNext()) {
                Object item = it.next();
                touches++;
                String text = (String) item;
                length = result + text.length() + item.toString().length();
            } else {
                return result;
            }
        }
    }

    public static int touchBeforeNext(Iterable values) {
        Iterator it = values.iterator();
        int length = 0;
        while (true) {
            int result = length;
            if (it.hasNext()) {
                touches++;
                Object item = it.next();
                String text = (String) item;
                length = result + text.length();
            } else {
                return result;
            }
        }
    }
}
