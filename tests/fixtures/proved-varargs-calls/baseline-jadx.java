package defpackage;

/* JADX INFO: loaded from: VarargsCalls.class */
public class VarargsCalls {
    static int effects;
    static java.lang.String order;

    static int mark(int i) {
        effects++;
        order += i;
        if (i == 9) {
            throw new java.lang.IllegalStateException("stop");
        }
        return i;
    }

    static int count(int... iArr) {
        return iArr.length;
    }

    static int objects(java.lang.Object... objArr) {
        return objArr.length;
    }

    static java.lang.String shape(java.lang.Object... objArr) {
        return objArr.getClass().getName() + "/" + objArr.length;
    }

    static int strings(java.lang.String... strArr) {
        return strArr.length;
    }

    static int plain(int[] iArr) {
        return iArr.length;
    }

    static int overloaded(java.lang.Object... objArr) {
        return objArr.length;
    }

    static int overloaded(java.lang.String str) {
        return str.length();
    }

    static int ordered() {
        return count(mark(1), mark(2), mark(3));
    }

    static int oneString() {
        return strings("one");
    }

    static int objectValues() {
        return objects("a", "b");
    }

    static int explicitVarargsArray() {
        return count(mark(4), mark(5));
    }

    static int ordinaryArray() {
        return plain(new int[]{mark(6), mark(7)});
    }

    static int heldArray() {
        return count(mark(8), mark(1));
    }

    static int overloadedArray() {
        return overloaded("x");
    }

    static java.lang.String nullElement() {
        return shape(null);
    }

    static java.lang.String arrayElement() {
        return shape("x");
    }

    static int exceptionOrder() {
        return count(mark(1), mark(9), mark(2));
    }
}
