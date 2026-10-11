package cf11;

import java.util.Iterator;
import java.util.List;

public final class HoistedIteratorRefusal {
    public static int sumWithWrittenGap(List<String> values) {
        int seen = 0;
        int total = 0;
        try {
            Iterator<String> iterator = values.iterator();
            seen = 1;
            while (iterator.hasNext()) {
                String value = iterator.next();
                total += value.length();
            }
        } catch (IllegalStateException error) {
            return seen;
        }
        return total + seen;
    }
}
