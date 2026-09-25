import java.util.Arrays;
import java.util.Iterator;

public class NonIterableCursor {
    static final class CursorBox {
        final String[] values;

        CursorBox(String... values) {
            this.values = values;
        }

        Iterator<String> iterator() {
            return Arrays.asList(values).iterator();
        }
    }

    static int sum(CursorBox box) {
        int length = 0;
        Iterator<String> it = box.iterator();
        while (it.hasNext()) {
            String value = it.next();
            length += value.length();
        }
        return length;
    }
}
