package cf11;

import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/HoistedIteratorPositive.class */
public final class HoistedIteratorPositive {
    public static int bodyCalls;

    public static int sumNested(List<String> outer, List<String> inner) {
        int total = 0;
        for (String left : outer) {
            for (String right : inner) {
                total += body(left, right);
            }
        }
        return total;
    }

    private static int body(String left, String right) {
        bodyCalls++;
        return left.length() + right.length();
    }
}
