import java.util.Collection;
import java.util.List;

public final class SubtypeOwners {
    public static int sumList(List<String> values) {
        int sum = 0;
        for (String value : values) sum += value.length();
        return sum;
    }

    public static int sumCollection(Collection<String> values) {
        int sum = 0;
        for (String value : values) sum += value.length();
        return sum;
    }

    public static int sumTextIterable(TextIterable values) {
        int sum = 0;
        for (String value : values) sum += value.length();
        return sum;
    }
}
