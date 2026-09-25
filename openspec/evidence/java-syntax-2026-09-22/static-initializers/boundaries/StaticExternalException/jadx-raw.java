package defpackage;

/* JADX INFO: loaded from: StaticExternalException.class */
public class StaticExternalException {
    static int value = ThrowingHelper.fail();

    public static int get() {
        return value;
    }
}
