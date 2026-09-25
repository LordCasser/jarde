import java.util.Arrays;

public final class ArrayInitializerBoundariesRunner {
    private interface Action {
        Object run() throws Throwable;
    }

    private static void show(String name, Action action) {
        try {
            Object value = action.run();
            System.out.println(name + "=" + (value != null && value.getClass().isArray()
                    ? arrayString(value) : String.valueOf(value)));
        } catch (Throwable error) {
            System.out.println(name + "!=" + error.getClass().getName());
        }
    }

    private static String arrayString(Object value) {
        if (value instanceof int[]) return Arrays.toString((int[]) value);
        if (value instanceof String[]) return Arrays.toString((String[]) value);
        if (value instanceof Object[]) return Arrays.toString((Object[]) value);
        throw new AssertionError(value.getClass());
    }

    private static Object escapedBeforeStores() {
        int[] value = ArrayInitializerBoundaries.escapedBeforeStores();
        if (ArrayInitializerBoundaries.escapedFirstAtPublication() != 0
                || !Arrays.equals(value, new int[] { 11, 22 })) {
            throw new AssertionError("escape observation or final array contents changed");
        }
        System.out.println("escaped-at-publication="
                + ArrayInitializerBoundaries.escapedFirstAtPublication());
        return value;
    }

    public static void main(String[] args) {
        show("exact", new Action() { public Object run() { return ArrayInitializerBoundaries.exactOrderedStores(); } });
        show("effectful", new Action() { public Object run() { return ArrayInitializerBoundaries.exactEffectfulStores(); } });
        show("reference", new Action() { public Object run() { return ArrayInitializerBoundaries.exactReferenceStores(); } });
        show("dynamic", new Action() { public Object run() { return ArrayInitializerBoundaries.dynamicLength(2); } });
        show("duplicate", new Action() { public Object run() { return ArrayInitializerBoundaries.duplicateIndex(); } });
        show("skipped", new Action() { public Object run() { return ArrayInitializerBoundaries.skippedIndex(); } });
        show("escaped", new Action() { public Object run() { return escapedBeforeStores(); } });
        show("branch-true", new Action() { public Object run() { return ArrayInitializerBoundaries.storesAcrossBranch(true); } });
        show("branch-false", new Action() { public Object run() { return ArrayInitializerBoundaries.storesAcrossBranch(false); } });
        show("covariant-store", new Action() { public Object run() { return ArrayInitializerBoundaries.covariantAastoreThrows(); } });
    }
}
