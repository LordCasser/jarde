package defpackage;

/* JADX INFO: loaded from: input.jar:Owner.class */
final class Owner {

    /* JADX INFO: renamed from: Owner$1, reason: invalid class name */
    /* JADX INFO: loaded from: input.jar:Owner$1.class */
    class AnonymousClass1 extends Base {
        AnonymousClass1() {
        }

        @Override // defpackage.Base
        int value() {
            return 1;
        }
    }

    Owner() {
    }

    static Base one() {
        return new AnonymousClass1();
    }
}
