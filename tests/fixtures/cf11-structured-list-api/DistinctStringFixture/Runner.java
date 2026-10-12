package cf11;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

public class Runner {
    private static ArrayList<String> list(String... values) {
        return new ArrayList<>(Arrays.asList(values));
    }

    private static String state(Object value) {
        return String.valueOf(value);
    }

    private static void run(String name, List<String> left, List<String> right) {
        DistinctStringFixture.effects = 0;
        Throwable failure = null;
        try {
            new DistinctStringFixture().renamed(left, right);
        } catch (Throwable error) {
            failure = error;
        }
        System.out.println(name
                + "|exception=" + (failure == null ? "none" : failure.getClass().getName())
                + "|left=" + state(left)
                + "|right=" + state(right)
                + "|effects=" + DistinctStringFixture.effects);
    }

    public static void main(String[] args) {
        run("string-isolated-empty", list(), list());
        run("string-isolated-add-cme", list("abcde", "later"), list("abcde", "tail"));
        run("string-isolated-remove-object-cme", list("four", "later", "tail"), list("four"));
        ArrayList<String> nullElement = new ArrayList<>();
        nullElement.add(null);
        run("string-isolated-null-element", nullElement, list("value"));
        run("string-isolated-null-list", null, list("value"));
    }
}
