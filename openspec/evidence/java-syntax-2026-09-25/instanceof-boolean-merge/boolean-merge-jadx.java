package defpackage;

/* JADX INFO: loaded from: BooleanMergeControls.class */
public final class BooleanMergeControls {
    static int calls;

    static java.lang.Object value(java.lang.Object obj) {
        calls++;
        return obj;
    }

    static boolean positive(java.lang.Object obj) {
        return value(obj) instanceof java.lang.String;
    }

    static boolean negative(java.lang.Object obj) {
        return !(value(obj) instanceof java.lang.String);
    }
}
