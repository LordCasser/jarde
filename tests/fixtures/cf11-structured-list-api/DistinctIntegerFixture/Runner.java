package cf11;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

public class Runner {
    private static ArrayList<Integer> list(Integer... values) {
        return new ArrayList<>(Arrays.asList(values));
    }

    private static String state(Object value) {
        return String.valueOf(value);
    }

    private static void run(String name, List<Integer> left, List<Integer> right) {
        DistinctIntegerFixture.effects = 0;
        Throwable failure = null;
        try {
            new DistinctIntegerFixture().numbers(left, right);
        } catch (Throwable error) {
            failure = error;
        }
        System.out.println(name
                + "|exception=" + (failure == null ? "none" : failure.getClass().getName())
                + "|left=" + state(left)
                + "|right=" + state(right)
                + "|effects=" + DistinctIntegerFixture.effects);
    }

    public static void main(String[] args) {
        run("integer-isolated-empty", list(), list());
        run("integer-isolated-add-cme", list(5, 8), list(5, 9));
        run("integer-isolated-remove-object-cme", list(4, 7, 8), list(4));
        ArrayList<Integer> nullElement = new ArrayList<>();
        nullElement.add(null);
        run("integer-isolated-null-element", nullElement, list(1));
        run("integer-isolated-null-list", null, list(1));
        List<Integer> polluted = list(1, 2);
        ((List) polluted).add("not-an-integer");
        run("integer-isolated-heap-pollution", polluted, list(9));
    }
}
