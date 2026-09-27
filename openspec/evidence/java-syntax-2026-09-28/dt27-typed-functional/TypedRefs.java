package dt27;

import java.util.function.Function;
import java.util.function.Supplier;

public class TypedRefs {
    private final String label;

    public TypedRefs(String label) {
        this.label = label;
    }

    public static Function<String, Integer> parse() {
        return Integer::parseInt;
    }

    public Function<String, Integer> bound() {
        return this::length;
    }

    public Supplier<String> supplier() {
        return this::label;
    }

    private Integer length(String text) {
        return text.length() + label.length();
    }

    private String label() {
        return label;
    }
}
