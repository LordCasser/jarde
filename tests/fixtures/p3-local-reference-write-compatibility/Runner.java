package cf11;

import java.util.AbstractList;
import java.util.Iterator;

@SuppressWarnings({"rawtypes", "unchecked"})
public final class Runner extends AbstractList implements Iterator {
    private static StringBuilder trace = new StringBuilder();

    private final String name;
    private final Object[] values;
    private final String throwAt;
    private int cursor;

    public Runner(String name, Object[] values, String throwAt) {
        this.name = name;
        this.values = values;
        this.throwAt = throwAt;
    }

    @Override
    public int size() {
        return values.length;
    }

    @Override
    public Object get(int index) {
        return values[index];
    }

    @Override
    public Iterator iterator() {
        trace.append(name).append(".iterator();");
        failAt("iterator");
        cursor = 0;
        return this;
    }

    @Override
    public boolean hasNext() {
        trace.append(name).append(".hasNext();");
        failAt("hasNext");
        return cursor < values.length;
    }

    @Override
    public Object next() {
        trace.append(name).append(".next();");
        failAt("next");
        return values[cursor++];
    }

    private void failAt(String phase) {
        boolean shouldThrow = phase.equals(throwAt);
        if ("hasNext-after-write".equals(throwAt)) {
            shouldThrow = "hasNext".equals(phase) && cursor > 0;
        }
        if (shouldThrow) {
            throw new IllegalStateException(name + "-" + phase);
        }
    }

    public static void assigned(String value) {
        trace.append("assigned=").append(String.valueOf(value)).append(';');
    }

    public static void beforeAssignment() {
        trace.append("before;");
    }

    private static Runner list(String name, Object... values) {
        return new Runner(name, values, null);
    }

    private static Runner throwing(String name, String phase, Object... values) {
        return new Runner(name, values, phase);
    }

    private static String exceptionName(Throwable error) {
        return error == null ? "none" : error.getClass().getName();
    }

    private static void run(String label, Runner values, int method) {
        trace = new StringBuilder();
        Object result = null;
        Throwable failure = null;
        try {
            switch (method) {
                case 0:
                    result = CF11OuterAssignmentTarget.positive(values);
                    break;
                case 1:
                    result = CF11OuterAssignmentTarget.noCastAssign(values);
                    break;
                case 2:
                    result = CF11OuterAssignmentTarget.bodyEffectBeforeAssign(values);
                    break;
                case 3:
                    result = CF11OuterAssignmentTarget.extraNextConsumption(values);
                    break;
                default:
                    throw new AssertionError("unknown method " + method);
            }
        } catch (Throwable error) {
            failure = error;
        }
        System.out.println(label + "|result=" + String.valueOf(result)
                + "|exception=" + exceptionName(failure)
                + "|trace=" + trace);
    }

    private static void runReferenceCase(String label, int method) {
        Object result = null;
        Throwable failure = null;
        try {
            switch (method) {
                case 0:
                    result = CF11CastReadControl.castRead(java.util.Arrays.asList("four"));
                    break;
                case 1:
                    result = CF11CharSequenceControl.stringLength();
                    break;
                case 2:
                    result = CF11CharSequenceControl.builderLength();
                    break;
                case 3:
                    result = CF11ReferenceArrayControl.lengthAt(false);
                    break;
                case 4:
                    result = CF11ReferenceArrayControl.lengthAt(true);
                    break;
                case 5:
                    result = CF11ParameterControl.replace("before", java.util.Arrays.asList("after"));
                    break;
                case 6:
                    result = CF11SlotReuseControl.afterStringLifetime(
                            "slot-reuse-object-write".equals(label),
                            java.util.Arrays.asList("replacement"));
                    break;
                case 7:
                    result = CF11HandlerWriteControl.handlerWrite();
                    break;
                default:
                    throw new AssertionError("unknown reference case " + method);
            }
        } catch (Throwable error) {
            failure = error;
        }
        System.out.println(label + "|result=" + String.valueOf(result)
                + "|exception=" + exceptionName(failure));
    }

    public static void main(String[] args) {
        run("positive-empty", list("values"), 0);
        run("positive-last-value", list("values", "first", "last"), 0);
        run("positive-null-element", list("values", "first", null), 0);
        run("positive-wrong-type-after-write",
                list("values", "last-good", Integer.valueOf(7)), 0);
        run("positive-null-iterable", null, 0);
        run("positive-iterator-throws", throwing("values", "iterator", "first"), 0);
        run("positive-hasNext-throws-before-write", throwing("values", "hasNext", "first"), 0);
        run("positive-next-throws-before-write", throwing("values", "next", "first"), 0);
        run("positive-hasNext-throws-after-write",
                throwing("values", "hasNext-after-write", "first", "second"), 0);

        runReferenceCase("cast-read", 0);
        runReferenceCase("char-sequence-string", 1);
        runReferenceCase("char-sequence-builder", 2);
        runReferenceCase("reference-array-string-array", 3);
        runReferenceCase("reference-array-interface-array", 4);
        runReferenceCase("parameter-write", 5);
        runReferenceCase("slot-reuse-no-replace", 6);
        runReferenceCase("slot-reuse-object-write", 6);
        runReferenceCase("handler-object-write", 7);

        run("control-no-cast-assign", list("values", "one", "two"), 1);
        run("control-no-cast-empty", list("values"), 1);
        run("control-no-cast-null-element", list("values", "first", null), 1);
        run("control-no-cast-integer-element",
                list("values", "last-good", Integer.valueOf(7)), 1);
        run("control-no-cast-null-iterable", null, 1);
        run("control-no-cast-iterator-throws", throwing("values", "iterator", "first"), 1);
        run("control-no-cast-hasNext-throws-before-write",
                throwing("values", "hasNext", "first"), 1);
        run("control-no-cast-next-throws-before-write",
                throwing("values", "next", "first"), 1);
        run("control-no-cast-hasNext-throws-after-write",
                throwing("values", "hasNext-after-write", "first", "second"), 1);
        run("control-body-effect-before-assign", list("values", "one"), 2);
        run("control-extra-next-consumption", list("values", "discard", "assigned"), 3);
    }
}
