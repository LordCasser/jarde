import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.function.Function;
import java.util.function.Supplier;
public class Y1M {
    static class Solo { int v() { return 7; } }
    static String viaLambda(String s) {
        Function<String, String> f = x -> x + "!";
        return f.apply(s);
    }
    static int viaMethodRef() {
        Supplier<Integer> sup = () -> 42;
        Function<String, Integer> len = String::length;
        return sup.get() + len.apply("hey");
    }
    static List<String> viaStream() {
        List<String> xs = new ArrayList<String>(Arrays.asList("b", "aa", "ccc"));
        xs.removeIf(x -> x.length() > 2);
        xs.sort((a, b) -> a.length() - b.length());
        return xs;
    }
    static int captureLambda(int n) {
        Function<Integer, Integer> add = v -> v + n;
        return add.apply(5);
    }
    public static void main(String[] a) {
        System.out.println(viaLambda("hi"));
        System.out.println(viaMethodRef());
        System.out.println(viaStream());
        System.out.println(captureLambda(3));
        System.out.println(new Solo().v());
    }
}
