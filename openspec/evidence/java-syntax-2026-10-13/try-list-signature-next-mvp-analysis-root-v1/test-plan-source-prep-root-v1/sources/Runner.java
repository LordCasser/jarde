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

        // The exact eight fixed CF11 input scenarios; preserve their output rows byte-for-byte.
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

        // Candidate traces. Keep these after the 17 frozen oracle rows above.
        final TraceList<String> sSuccessIn = traceStrings("input", "positive", "");
        final TraceList<String> sSuccessOut = traceStrings("output");
        runTrace("try-string-success-both-branches", () -> subject.tryStringBranch(sSuccessIn, sSuccessOut), sSuccessIn, sSuccessOut);
        final TraceList<String> sEmptyIn = traceStrings("input");
        final TraceList<String> sEmptyOut = traceStrings("output");
        runTrace("try-string-empty", () -> subject.tryStringBranch(sEmptyIn, sEmptyOut), sEmptyIn, sEmptyOut);
        final TraceList<String> sNullIn = traceStrings("input", (String) null);
        final TraceList<String> sNullOut = traceStrings("output");
        runTrace("try-string-null-element", () -> subject.tryStringBranch(sNullIn, sNullOut), sNullIn, sNullOut);

        final TraceList<Integer> iSuccessIn = traceIntegers("input", 3, -1);
        final TraceList<Integer> iSuccessOut = traceIntegers("output");
        runTrace("try-integer-success-both-branches", () -> subject.tryIntegerBranch(iSuccessIn, iSuccessOut), iSuccessIn, iSuccessOut);
        final TraceList<Integer> iEmptyIn = traceIntegers("input");
        final TraceList<Integer> iEmptyOut = traceIntegers("output");
        runTrace("try-integer-empty", () -> subject.tryIntegerBranch(iEmptyIn, iEmptyOut), iEmptyIn, iEmptyOut);
        final TraceList<Integer> iNullIn = traceIntegers("input", (Integer) null);
        final TraceList<Integer> iNullOut = traceIntegers("output");
        runTrace("try-integer-null-element", () -> subject.tryIntegerBranch(iNullIn, iNullOut), iNullIn, iNullOut);

        final TraceList<String> nestedIn = traceStrings("input", "a", "b");
        final TraceList<String> nestedWitness = traceStrings("witness", "b");
        final TraceList<String> nestedOut = traceStrings("output");
        runTrace("try-string-nested-pending-topology", () -> subject.tryStringNested(nestedIn, nestedWitness, nestedOut), nestedIn, nestedOut);
        final TraceList<String> unknownIn = traceStrings("input", "a");
        final TraceList<String> unknownOut = traceStrings("output");
        runTrace("try-string-unknown-call-control", () -> subject.tryStringUnknownCall(unknownIn, unknownOut), unknownIn, unknownOut);

        for (String point : new String[] {"iterator", "hasNext", "next", "add"}) {
            for (String kind : new String[] {"runtime", "error"}) {
                runInjected("string-" + point + "-" + kind, subject, point, kind,
                        strings("one", "two"), point.equals("add"),
                        (input, output) -> subject.tryStringBranch(input, output));
                runInjected("integer-" + point + "-" + kind, subject, point, kind,
                        integers(1, 2), point.equals("add"),
                        (input, output) -> subject.tryIntegerBranch(input, output));
            }
        }

        final TraceList<String> prefixInput = traceStrings("input", "x");
        final TraceList<String> prefixOutput = traceStrings("output", "key", "old");
        runTrace("remove-object-prefix-normal", () -> subject.tryStringRemovePrefix(prefixInput, prefixOutput, "key"), prefixInput, prefixOutput);
        runPrefixFailure(subject, "runtime");
        runPrefixFailure(subject, "error");
    }

    private static TraceList<String> traceStrings(String label, String... values) {
        TraceList<String> result = new TraceList<>(label);
        for (String value : values) result.seed(value);
        return result;
    }

    private static TraceList<Integer> traceIntegers(String label, Integer... values) {
        TraceList<Integer> result = new TraceList<>(label);
        for (Integer value : values) result.seed(value);
        return result;
    }

    private static final List<String> EVENTS = new ArrayList<>();

    private static final class TraceList<E> extends ArrayList<E> {
        private final String label;
        private String failAt;
        private String failKind;
        private boolean failed;

        TraceList(String label) {
            this.label = label;
        }

        void seed(E value) {
            super.add(value);
        }

        void inject(String operation, String kind) {
            this.failAt = operation;
            this.failKind = kind;
            this.failed = false;
        }

        private void step(String operation) {
            String event = label + "." + operation;
            EVENTS.add(event);
            if (!failed && operation.equals(failAt)) {
                failed = true;
                if ("error".equals(failKind)) {
                    throw new AssertionError("injected:" + event);
                }
                throw new IllegalStateException("injected:" + event);
            }
        }

        @Override
        public java.util.Iterator<E> iterator() {
            step("iterator");
            final java.util.Iterator<E> delegate = super.iterator();
            return new java.util.Iterator<E>() {
                @Override public boolean hasNext() { step("hasNext"); return delegate.hasNext(); }
                @Override public E next() { step("next"); return delegate.next(); }
            };
        }

        @Override public boolean add(E value) { step("add"); return super.add(value); }
        @Override public boolean remove(Object value) { step("removeObject"); return super.remove(value); }
        @Override public E remove(int index) { step("removeIndex"); return super.remove(index); }
        @Override public int size() { step("size"); return super.size(); }
        @Override public void clear() { step("clear"); super.clear(); }

        String snapshot() { return Arrays.toString(super.toArray()); }
    }

    private interface TraceAction<E> {
        void run(TraceList<E> input, TraceList<E> output);
    }

    private static <E> void runInjected(String name, UpstreamNestedLoops subject,
            String operation, String kind, List<E> values, boolean failOnOutput, TraceAction<E> action) {
        TraceList<E> input = new TraceList<>("input");
        for (E value : values) input.seed(value);
        TraceList<E> output = new TraceList<>("output");
        (failOnOutput ? output : input).inject(operation, kind);
        runTrace(name, () -> action.run(input, output), input, output);
    }

    private static void runPrefixFailure(UpstreamNestedLoops subject, String kind) {
        TraceList<String> input = traceStrings("input", "x");
        TraceList<String> output = traceStrings("output", "key", "old");
        output.inject("removeObject", kind);
        runTrace("remove-object-prefix-" + kind, () -> subject.tryStringRemovePrefix(input, output, "key"), input, output);
    }

    private static void runTrace(String name, Runnable action, TraceList<?> input, TraceList<?> output) {
        EVENTS.clear();
        UpstreamNestedLoops.effects = 0;
        Throwable failure = null;
        try { action.run(); } catch (Throwable error) { failure = error; }
        System.out.println(name + "|exception=" + (failure == null ? "none" : failure.getClass().getName())
                + ":" + (failure == null ? "" : failure.getMessage())
                + "|input=" + input.snapshot() + "|output=" + output.snapshot()
                + "|effects=" + UpstreamNestedLoops.effects + "|events=" + EVENTS);
    }
}
