import java.util.Iterator;

public final class RawIterableIterator {
    public static int touches;

    public static int rawCast(Iterable values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            Object item = it.next();
            String text = (String) item;
            result += text.length();
        }
        return result;
    }

    public static int nextTwiceCastBeforeTouch(Iterable values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            Object item = it.next();
            String text = (String) item;
            touches++;
            result += text.length() + item.toString().length();
        }
        return result;
    }

    public static int nextTwiceTouchBeforeCast(Iterable values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            Object item = it.next();
            touches++;
            String text = (String) item;
            result += text.length() + item.toString().length();
        }
        return result;
    }

    public static int touchBeforeNext(Iterable values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            touches++;
            Object item = it.next();
            String text = (String) item;
            result += text.length();
        }
        return result;
    }
}
