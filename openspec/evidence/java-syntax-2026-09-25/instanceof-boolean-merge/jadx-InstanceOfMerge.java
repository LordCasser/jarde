package defpackage;

/* JADX INFO: loaded from: InstanceOfMerge.class */
public final class InstanceOfMerge {
    static int calls;

    static java.lang.Object value(java.lang.Object obj) {
        calls++;
        return obj;
    }

    static boolean inverted(java.lang.Object obj) {
        return !(value(obj) instanceof java.lang.String);
    }

    static boolean direct(java.lang.Object obj) {
        return value(obj) instanceof java.lang.String;
    }
}
