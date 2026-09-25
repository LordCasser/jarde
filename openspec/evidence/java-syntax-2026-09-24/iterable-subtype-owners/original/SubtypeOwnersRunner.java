import java.util.Arrays;
import java.util.Collection;
import java.util.List;

public final class SubtypeOwnersRunner {
    public static void main(String[] args) {
        List<String> values = Arrays.asList("a", "bc", "def");
        Collection<String> collection = values;
        TextIterable custom = values::iterator;
        System.out.println("list=" + SubtypeOwners.sumList(values));
        System.out.println("collection=" + SubtypeOwners.sumCollection(collection));
        System.out.println("textIterable=" + SubtypeOwners.sumTextIterable(custom));
    }
}
