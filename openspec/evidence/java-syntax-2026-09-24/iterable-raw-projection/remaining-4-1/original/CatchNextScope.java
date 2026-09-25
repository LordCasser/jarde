import java.util.Iterator;

public final class CatchNextScope {
    public static int consume(Iterable values) {
        Iterator it = values.iterator();
        int result = 0;
        while (it.hasNext()) {
            try {
                Object item = it.next();
                String text = (String) item;
                result += text.length();
            } catch (IllegalStateException e) {
                result++;
            }
        }
        return result;
    }
}
