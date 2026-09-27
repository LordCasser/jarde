package defpackage;

import java.util.Arrays;

/* JADX INFO: loaded from: ForeachCases.class */
public class ForeachCases {
    public static int sum(int[] values) {
        int total = 0;
        for (int value : values) {
            total += value;
        }
        return total;
    }

    public static String join(Iterable<String> values) {
        StringBuilder result = new StringBuilder();
        for (String value : values) {
            result.append(value);
        }
        return result.toString();
    }

    public static int everyOther(int[] values) {
        int total = 0;
        for (int i = 0; i < values.length; i += 2) {
            total += values[i];
        }
        return total;
    }

    public static void main(String[] args) {
        System.out.println(sum(new int[]{1, 2, 3, 4}));
        System.out.println(join(Arrays.asList("a", "b", "c")));
        System.out.println(everyOther(new int[]{1, 2, 3, 4}));
    }
}
