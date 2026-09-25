package defpackage;

/* JADX INFO: loaded from: BoundarySupport.class */
final class BoundarySupport {
    static int calls;

    BoundarySupport() {
    }

    static void constructorEffect() {
        calls++;
    }

    static int prefixValue(int i) {
        calls++;
        return i;
    }
}
