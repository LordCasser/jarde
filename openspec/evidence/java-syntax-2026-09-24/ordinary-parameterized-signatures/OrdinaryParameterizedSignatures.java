import java.util.*;

public class OrdinaryParameterizedSignatures {
    public static Iterable<String> strings(Iterable<String> input) { return input; }
    public static List<? extends Number> numbers(List<? extends Number> input) { return input; }
    public static Map<String, List<Integer>> nested(Map<String, List<Integer>> input) { return input; }
    public static List<String>[] arrays(List<String>[] input) { return input; }
    @SuppressWarnings("rawtypes") public static List raw(List input) { return input; }

}
