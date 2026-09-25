package defpackage;

/* JADX INFO: loaded from: CtorOverloads.class */
public class CtorOverloads {
    private final int code = 1;

    public CtorOverloads(Object obj) {
    }

    public CtorOverloads(String str) {
    }

    public static int run() {
        return new CtorOverloads((Object) null).code;
    }
}
