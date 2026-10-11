package cf11;

import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/HoistedIteratorRefusal.class */
public final class HoistedIteratorRefusal {
    public static int sumWithWrittenGap(List<String> values) {
        int seen = 0;
        int total = 0;
        try {
            seen = 1;
            for (String value : values) {
                total += value.length();
            }
            return total + 1;
        } catch (IllegalStateException e) {
            return seen;
        }
    }
}
