package defpackage;

/* JADX INFO: loaded from: fam.jar:F1.class */
public class F1 {

    /* JADX INFO: loaded from: fam.jar:F1$A.class */
    interface A {
        default String name() {
            return "A";
        }
    }

    /* JADX INFO: loaded from: fam.jar:F1$B.class */
    interface B {
        default String name() {
            return "B";
        }
    }

    /* JADX INFO: loaded from: fam.jar:F1$Diamond.class */
    static class Diamond implements A, B {
        Diamond() {
        }

        @Override // F1.A, F1.B
        public String name() {
            return super.name() + super.name();
        }
    }

    /* JADX INFO: loaded from: fam.jar:F1$Reabstract.class */
    static class Reabstract implements A {

        /* JADX INFO: loaded from: fam.jar:F1$Reabstract$C.class */
        interface C extends A {
            @Override // F1.A, F1.B
            String name();
        }

        /* JADX INFO: loaded from: fam.jar:F1$Reabstract$Impl.class */
        static class Impl implements A {
            Impl() {
            }

            @Override // F1.A, F1.B
            public String name() {
                return "I:" + super.name();
            }
        }

        Reabstract() {
        }
    }

    static String diamond() {
        return new Diamond().name();
    }

    static String reabstract() {
        return new Reabstract.Impl().name();
    }

    public static void main(String[] strArr) {
        System.out.println(diamond());
        System.out.println(reabstract());
    }
}
