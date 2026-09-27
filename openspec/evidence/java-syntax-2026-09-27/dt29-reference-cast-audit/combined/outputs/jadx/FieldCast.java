package dt29;

/* JADX INFO: loaded from: input.jar:dt29/FieldCast.class */
public class FieldCast {

    /* JADX INFO: loaded from: input.jar:dt29/FieldCast$A.class */
    public static class A {
        public boolean publicField;
        boolean packagePrivateField;
        protected boolean protectedField;
        private boolean privateField;
    }

    /* JADX INFO: loaded from: input.jar:dt29/FieldCast$B.class */
    public static class B extends A {
        public void self(boolean z) {
            this.publicField = z;
            this.protectedField = z;
            this.packagePrivateField = z;
            ((A) this).privateField = z;
        }
    }

    /* JADX INFO: loaded from: input.jar:dt29/FieldCast$C.class */
    public static class C {
        public void set(B b, boolean z) {
            b.publicField = z;
            b.protectedField = z;
            b.packagePrivateField = z;
            ((A) b).privateField = z;
        }
    }

    /* JADX INFO: loaded from: input.jar:dt29/FieldCast$D.class */
    private static class D {
        private D() {
        }

        public <T extends B> void set(T t, boolean z) {
            t.publicField = z;
            t.protectedField = z;
            t.packagePrivateField = z;
            ((A) t).privateField = z;
        }
    }

    public static String run() {
        B b = new B();
        b.self(true);
        String strBits = bits(b);
        new C().set(b, false);
        String strBits2 = bits(b);
        new D().set(b, true);
        return strBits + ":" + strBits2 + ":" + bits(b);
    }

    private static String bits(A a) {
        return (a.publicField ? "1" : "0") + (a.protectedField ? "1" : "0") + (a.packagePrivateField ? "1" : "0") + (a.privateField ? "1" : "0");
    }
}
