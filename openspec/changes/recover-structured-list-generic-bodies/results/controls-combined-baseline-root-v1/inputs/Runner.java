package cf11;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;

public class Runner {
    private static ArrayList<String> strings(String... values) {
        return new ArrayList<>(Arrays.asList(values));
    }

    private static ArrayList<Integer> integers(Integer... values) {
        return new ArrayList<>(Arrays.asList(values));
    }

    private static String state(Object value) {
        return String.valueOf(value);
    }

    private static void run(String name, Runnable action, Object left, Object right) {
        UpstreamNestedLoops.effects = 0;
        Throwable failure = null;
        try {
            action.run();
        } catch (Throwable error) {
            failure = error;
        }
        System.out.println(name
                + "|exception=" + (failure == null ? "none" : failure.getClass().getName())
                + "|left=" + state(left)
                + "|right=" + state(right)
                + "|effects=" + UpstreamNestedLoops.effects);
    }

    public static void main(String[] args) {
        final UpstreamNestedLoops subject = new UpstreamNestedLoops();

        // The exact eight fixed CF11 input scenarios.
        final List<String> a0 = strings();
        final List<String> b0 = strings();
        run("baseline-both-empty", () -> subject.test(a0, b0), a0, b0);
        final List<String> a1 = strings("left");
        final List<String> b1 = strings("right");
        run("baseline-unmatched", () -> subject.test(a1, b1), a1, b1);
        final List<String> a2 = strings("abcde", "later");
        final List<String> b2 = strings("abcde", "tail");
        run("baseline-five-character-match-add", () -> subject.test(a2, b2), a2, b2);
        final List<String> a3 = strings("four", "later", "tail");
        final List<String> b3 = strings("four");
        run("baseline-non-five-match-remove", () -> subject.test(a3, b3), a3, b3);
        final List<String> a4 = strings("four", "keep");
        final List<String> b4 = strings("four");
        run("baseline-remove-continues-and-clears", () -> subject.test(a4, b4), a4, b4);
        final List<String> b5 = strings("value");
        run("baseline-null-first-list", () -> subject.test(null, b5), null, b5);
        final List<String> a6 = strings("value");
        run("baseline-null-second-list", () -> subject.test(a6, null), a6, null);
        final List<String> a7 = new ArrayList<>();
        a7.add(null);
        final List<String> b7 = strings("value");
        run("baseline-null-element", () -> subject.test(a7, b7), a7, b7);

        final List<String> s0 = strings("alpha", "later");
        final List<String> s1 = strings("alpha", "tail");
        run("renamed-string-positive", () -> subject.renamed(s0, s1), s0, s1);
        final List<Integer> n0 = integers(5, 8);
        final List<Integer> n1 = integers(5, 9);
        run("integer-nested-add-positive", () -> subject.numbers(n0, n1), n0, n1);
        final List<String> c0 = strings("same");
        final List<String> c1 = strings("same");
        run("explicit-cast-object-boundary", () -> subject.castBoundary(c0, c1), c0, c1);

        final List rawLeft = new ArrayList(Arrays.asList("one", "two"));
        final List rawRight = new ArrayList(Arrays.asList("two"));
        run("raw-list-method-control", () -> subject.rawLists(rawLeft, rawRight), rawLeft, rawRight);
        final List<String> aliasTarget = strings("kept");
        final List<Integer> aliasSource = integers(7);
        run("raw-alias-heap-pollution", () -> subject.rawAliasEscape(aliasTarget, aliasSource), aliasTarget, aliasSource);
        final List<String> originalInput = strings("original");
        final List<String> replacement = strings("replacement");
        run("parameter-reassignment", () -> subject.reassigned(originalInput, replacement), originalInput, replacement);
        final List<Number> wildcardValues = new ArrayList<Number>(Arrays.<Number>asList(3));
        final List<Number> wildcardSink = new ArrayList<Number>();
        run("wildcard-null-write", () -> subject.wildcardNull(wildcardValues, wildcardSink), wildcardValues, wildcardSink);
        final List<String> handlerInput = strings("one", "two");
        final List<String> handlerOutput = Collections.unmodifiableList(new ArrayList<String>());
        run("try-catch-handler", () -> subject.handled(handlerInput, handlerOutput), handlerInput, handlerOutput);
        final List<Integer> overloads = integers(4, 5, 6);
        final List<Integer> overloadWitness = integers(9);
        run("remove-object-and-index-overloads", () -> subject.indexOverload(overloads, overloadWitness, 5, 1), overloads, overloadWitness);
    }
}
