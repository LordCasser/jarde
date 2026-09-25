package defpackage;

/* JADX INFO: loaded from: input.jar:Other.class */
final class Other {

    /* JADX INFO: renamed from: Other$1, reason: invalid class name */
    /* JADX INFO: loaded from: input.jar:Other$1.class */
    class AnonymousClass1 extends Base {
        AnonymousClass1() {
        }

        @Override // defpackage.Base
        int value() {
            return 2;
        }
    }

    Other() {
    }

    static Base two() {
        return new Owner.AnonymousClass1();
    }
}
