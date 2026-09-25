package defpackage;

/* JADX INFO: loaded from: FakeBridge.class */
public final class FakeBridge {
    private static int calls;

    public String get() {
        return "value";
    }

    /* JADX INFO: renamed from: get, reason: collision with other method in class */
    public /* bridge */ /* synthetic */ Object m0get() {
        calls++;
        return get();
    }

    public static int count() {
        return calls;
    }
}
