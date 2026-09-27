import java.util.Arrays;

public class ListToIterable {
    public static void consume(Iterable values) {
        System.out.println("called");
    }

    public static void main(String[] args) {
        consume(Arrays.asList("a", "b", "c"));
    }
}
