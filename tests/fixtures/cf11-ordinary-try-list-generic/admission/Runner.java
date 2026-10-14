package cf11;

import java.util.ArrayList;
import java.util.Arrays;
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
        OrdinaryTryListFixture.effects = 0;
        Throwable failure = null;
        try {
            action.run();
        } catch (Throwable error) {
            failure = error;
        }
        System.out.println(name + "|exception=" + (failure == null ? "none" : failure.getClass().getName())
                + "|left=" + state(left) + "|right=" + state(right)
                + "|effects=" + OrdinaryTryListFixture.effects);
    }

    private static void runTrace(String name, Runnable action, TraceList<?> input, TraceList<?> output) {
        EVENTS.clear();
        OrdinaryTryListFixture.effects = 0;
        Throwable failure = null;
        try {
            action.run();
        } catch (Throwable error) {
            failure = error;
        }
        System.out.println(name + "|exception="
                + (failure == null ? "none" : failure.getClass().getName())
                + ":" + (failure == null ? "" : failure.getMessage())
                + "|input=" + input.snapshot() + "|output=" + output.snapshot()
                + "|effects=" + OrdinaryTryListFixture.effects + "|events=" + EVENTS);
    }

    public static void main(String[] args) {
        final OrdinaryTryListFixture subject = new OrdinaryTryListFixture();

        List<String> stringInput = strings("", "text");
        List<String> stringOutput = strings();
        run("string-branch-normal", () -> subject.stringBranch(stringInput, stringOutput), stringInput, stringOutput);
        List<String> emptyStringInput = strings();
        List<String> emptyStringOutput = strings();
        run("string-empty", () -> subject.stringBranch(emptyStringInput, emptyStringOutput), emptyStringInput, emptyStringOutput);
        List<String> nullStringOutput = strings();
        run("string-null-list", () -> subject.stringBranch(null, nullStringOutput), null, nullStringOutput);
        List<String> nullElementInput = strings((String) null);
        List<String> nullElementOutput = strings();
        run("string-null-element", () -> subject.stringBranch(nullElementInput, nullElementOutput), nullElementInput, nullElementOutput);

        List<String> renamedSource = strings("", "text");
        List<String> renamedSink = strings();
        run("renamed-string-branch", () -> subject.renamedTry(renamedSource, renamedSink), renamedSource, renamedSink);

        List<Integer> integerInput = integers(3, -1);
        List<Integer> integerOutput = integers();
        run("integer-branches", () -> subject.integerTry(integerInput, integerOutput), integerInput, integerOutput);
        List<Integer> emptyIntegerInput = integers();
        List<Integer> emptyIntegerOutput = integers();
        run("integer-empty", () -> subject.integerTry(emptyIntegerInput, emptyIntegerOutput), emptyIntegerInput, emptyIntegerOutput);
        List<Integer> nullIntegerOutput = integers();
        run("integer-null-list", () -> subject.integerTry(null, nullIntegerOutput), null, nullIntegerOutput);
        List<Integer> nullIntegerInput = integers((Integer) null);
        List<Integer> nullIntegerElementOutput = integers();
        run("integer-null-element", () -> subject.integerTry(nullIntegerInput, nullIntegerElementOutput), nullIntegerInput, nullIntegerElementOutput);

        List<String> nestedSource = strings("a", "b");
        List<String> nestedWitness = strings("b");
        List<String> nestedDestination = strings();
        run("nested-foreach", () -> subject.nestedTry(nestedSource, nestedWitness, nestedDestination), nestedSource, nestedDestination);

        TraceList<String> prefixInput = traceStrings("input", "new");
        TraceList<String> prefixOutput = traceStrings("output", "key", "old");
        runTrace("remove-object-prefix-normal",
                () -> subject.stringRemovePrefix(prefixInput, prefixOutput, "key"), prefixInput, prefixOutput);
        runPrefixFailure(subject, "runtime");
        runPrefixFailure(subject, "error");

        List<Integer> overloadValues = integers(4, 5, 6);
        List<Integer> overloadWitness = integers();
        run("remove-object-and-index-control", () -> subject.indexOverload(overloadValues, overloadWitness, 5, 1), overloadValues, overloadWitness);

        for (String point : new String[] {"iterator", "hasNext", "next", "add"}) {
            for (String kind : new String[] {"runtime", "error"}) {
                runInjected("string-" + point + "-" + kind, subject, point, kind,
                        strings("one", "two"), point.equals("add"),
                        (input, output) -> subject.stringBranch(input, output));
                runInjected("integer-" + point + "-" + kind, subject, point, kind,
                        integers(1, 2), point.equals("add"),
                        (input, output) -> subject.integerTry(input, output));
            }
        }

        TraceList<String> gapInput = traceStrings("input", "one");
        TraceList<String> gapOutput = traceStrings("output");
        runTrace("iterator-gap-normal", () -> subject.catchVisibleIteratorGap(gapInput, gapOutput),
                gapInput, gapOutput);
        runIteratorGapFailure(subject, "runtime");
        runIteratorGapFailure(subject, "error");
    }

    private static final List<String> EVENTS = new ArrayList<>();

    private static final class TraceList<E> extends ArrayList<E> {
        private final String label;
        private String failAt;
        private String failKind;
        private boolean failed;

        TraceList(String label) { this.label = label; }

        void seed(E value) { super.add(value); }

        void inject(String operation, String kind) {
            failAt = operation;
            failKind = kind;
            failed = false;
        }

        private void step(String operation) {
            String event = label + "." + operation;
            EVENTS.add(event);
            if (!failed && operation.equals(failAt)) {
                failed = true;
                if ("error".equals(failKind)) throw new AssertionError("injected:" + event);
                throw new IllegalStateException("injected:" + event);
            }
        }

        @Override public java.util.Iterator<E> iterator() {
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

    private static TraceList<String> traceStrings(String label, String... values) {
        TraceList<String> result = new TraceList<>(label);
        for (String value : values) result.seed(value);
        return result;
    }

    private static void runPrefixFailure(OrdinaryTryListFixture subject, String kind) {
        TraceList<String> input = traceStrings("input", "new");
        TraceList<String> output = traceStrings("output", "key", "old");
        output.inject("removeObject", kind);
        EVENTS.clear();
        OrdinaryTryListFixture.effects = 0;
        Throwable failure = null;
        try { subject.stringRemovePrefix(input, output, "key"); }
        catch (Throwable error) { failure = error; }
        System.out.println("remove-object-prefix-" + kind + "|exception="
                + (failure == null ? "none" : failure.getClass().getName())
                + ":" + (failure == null ? "" : failure.getMessage())
                + "|input=" + input.snapshot() + "|output=" + output.snapshot()
                + "|effects=" + OrdinaryTryListFixture.effects + "|events=" + EVENTS);
    }

    private static void runIteratorGapFailure(OrdinaryTryListFixture subject, String kind) {
        TraceList<String> input = traceStrings("input", "one");
        TraceList<String> output = traceStrings("output");
        input.inject("iterator", kind);
        EVENTS.clear();
        OrdinaryTryListFixture.effects = 0;
        Throwable failure = null;
        try { subject.catchVisibleIteratorGap(input, output); }
        catch (Throwable error) { failure = error; }
        System.out.println("iterator-gap-" + kind + "|exception="
                + (failure == null ? "none" : failure.getClass().getName())
                + ":" + (failure == null ? "" : failure.getMessage())
                + "|input=" + input.snapshot() + "|output=" + output.snapshot()
                + "|effects=" + OrdinaryTryListFixture.effects + "|events=" + EVENTS);
    }

    private static <E> void runInjected(String name, OrdinaryTryListFixture subject,
            String operation, String kind, List<E> values, boolean failOnOutput, TraceAction<E> action) {
        TraceList<E> input = new TraceList<>("input");
        for (E value : values) input.seed(value);
        TraceList<E> output = new TraceList<>("output");
        (failOnOutput ? output : input).inject(operation, kind);
        EVENTS.clear();
        OrdinaryTryListFixture.effects = 0;
        Throwable failure = null;
        try { action.run(input, output); } catch (Throwable error) { failure = error; }
        System.out.println(name + "|exception=" + (failure == null ? "none" : failure.getClass().getName())
                + ":" + (failure == null ? "" : failure.getMessage())
                + "|input=" + input.snapshot() + "|output=" + output.snapshot()
                + "|effects=" + OrdinaryTryListFixture.effects + "|events=" + EVENTS);
    }
}
