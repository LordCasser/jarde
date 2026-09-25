package defpackage;

/* JADX INFO: loaded from: input.jar:AnonymousDoubleSite.class */
public final class AnonymousDoubleSite {
    static DoubleBase create(boolean first) {
        if (first) {
            return new DoubleBase() { // from class: AnonymousDoubleSite.1
                @Override // defpackage.DoubleBase
                int value() {
                    return 11;
                }
            };
        }
        return new DoubleBase() { // from class: AnonymousDoubleSite.1
            @Override // defpackage.DoubleBase
            int value() {
                return 11;
            }
        };
    }

    /* JADX INFO: renamed from: AnonymousDoubleSite$2, reason: invalid class name */
    /* JADX INFO: loaded from: input.jar:AnonymousDoubleSite$2.class */
    class AnonymousClass2 extends DoubleBase {
        AnonymousClass2() {
        }

        @Override // defpackage.DoubleBase
        int value() {
            return 22;
        }
    }

    public static void main(String[] args) {
        DoubleBase first = create(true);
        DoubleBase second = create(false);
        System.out.println("first=" + first.value());
        System.out.println("second=" + second.value());
        System.out.println("sameClass=" + (first.getClass() == second.getClass()));
    }
}
