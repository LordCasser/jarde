package defpackage;

/* JADX INFO: loaded from: GenericMethodProbe.class */
public class GenericMethodProbe {
    public static <T extends Number> T choose(T t, T t2, boolean z) {
        return z ? t : t2;
    }
}
