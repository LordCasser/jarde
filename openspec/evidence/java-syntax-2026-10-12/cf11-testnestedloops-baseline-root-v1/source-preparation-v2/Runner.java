package cf11;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

public class Runner {
    private static ArrayList<String> list(String... values) {
        return new ArrayList<>(Arrays.asList(values));
    }

    private static void run(String name, List<String> l1, List<String> l2) {
        Throwable failure = null;
        try {
            new UpstreamNestedLoops().test(l1, l2);
        } catch (Throwable error) {
            failure = error;
        }
        System.out.println(name
                + "|exception=" + (failure == null ? "none" : failure.getClass().getName())
                + "|l1=" + l1
                + "|l2=" + l2);
    }

    public static void main(String[] args) {
        run("both-empty", list(), list());
        run("unmatched", list("left"), list("right"));
        run("five-character-match-add", list("abcde", "later"), list("abcde", "tail"));
        run("non-five-match-remove", list("four", "later", "tail"), list("four"));
        run("non-five-match-continues-and-clears", list("keep", "four"), list("four"));
        run("null-first-list", null, list("value"));
        run("null-second-list", list("value"), null);

        ArrayList<String> nullElement = new ArrayList<>();
        nullElement.add(null);
        run("null-element", nullElement, list("value"));
    }
}
