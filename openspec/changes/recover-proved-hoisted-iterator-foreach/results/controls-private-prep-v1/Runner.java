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
        if (phase.equals(throwAt)) {
            throw new IllegalStateException(name + "-" + phase);
        }
    }

    private static Runner list(String name, String... values) {
        return new Runner(name, values, null);
    }

    private static Runner throwing(String name, String phase, Object... values) {
        return new Runner(name, values, phase);
    }

    private static void runPositive(String label, Runner outer, Runner inner) {
        trace = new StringBuilder();
        HoistedIteratorPositive.bodyCalls = 0;
        Throwable failure = null;
        int result = -1;
        try {
            result = HoistedIteratorPositive.sumNested(outer, inner);
        } catch (Throwable error) {
            failure = error;
        }
        System.out.println(label + "|result=" + result
                + "|bodyCalls=" + HoistedIteratorPositive.bodyCalls
                + "|exception=" + exceptionName(failure)
                + "|trace=" + trace);
    }

    private static void runRefusal(String label, Runner values) {
        trace = new StringBuilder();
        Throwable failure = null;
        int result = -1;
        try {
            result = HoistedIteratorRefusal.sumWithWrittenGap(values);
        } catch (Throwable error) {
            failure = error;
        }
        System.out.println(label + "|result=" + result
                + "|exception=" + exceptionName(failure)
                + "|trace=" + trace);
    }

    private static String exceptionName(Throwable error) {
        return error == null ? "none" : error.getClass().getName();
    }

    public static void main(String[] args) {
        runPositive("nested-two-by-two", list("outer", "A", "bb"), list("inner", "x", "yyy"));
        runPositive("outer-empty", list("outer"), list("inner", "x"));
        runPositive("inner-empty-reentered", list("outer", "A", "bb"), list("inner"));
        runPositive("null-outer", null, list("inner", "x"));
        runPositive("null-inner", list("outer", "A"), null);
        runPositive("throw-inner-iterator", list("outer", "A", "bb"), throwing("inner", "iterator", "x"));
        runPositive("throw-inner-hasNext", list("outer", "A"), throwing("inner", "hasNext", "x"));
        runPositive("throw-inner-next", list("outer", "A"), throwing("inner", "next", "x"));
        runPositive("throw-outer-iterator", throwing("outer", "iterator", "A"), list("inner", "x"));
        runPositive("throw-outer-hasNext", throwing("outer", "hasNext", "A"), list("inner", "x"));
        runPositive("throw-outer-next", throwing("outer", "next", "A"), list("inner", "x"));
        runPositive("wrong-element-cast", list("outer", "A"), new Runner("inner", new Object[] {"ok", Integer.valueOf(7)}, null));

        runRefusal("written-gap-normal", list("values", "one", "two"));
        runRefusal("written-gap-iterator-throws-before-write", throwing("values", "iterator", "one"));
        runRefusal("written-gap-hasNext-throws-after-write", throwing("values", "hasNext", "one"));
    }
}
