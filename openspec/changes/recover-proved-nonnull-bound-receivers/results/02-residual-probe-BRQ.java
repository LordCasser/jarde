import java.util.*;
public class BRQ {
    static java.util.function.Supplier<Integer> size() {
        ArrayList<String> out = new ArrayList<>();
        return out::size;
    }
    static java.util.function.Function<Object, Boolean> eq() {
        ArrayList<String> out = new ArrayList<>();
        return out::equals;
    }
    static java.util.function.Consumer<String> adder() {
        ArrayList<String> out = new ArrayList<>();
        return out::add;
    }
}
