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

    public static Object seed() {
        return "seed";
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

        run("control-no-cast-assign", list("values", "one", "two"), 1);
        run("control-body-effect-before-assign", list("values", "one"), 2);
        run("control-extra-next-consumption", list("values", "discard", "assigned"), 3);
    }
}
