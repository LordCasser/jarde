package dt29;

/* JADX INFO: loaded from: private-field-input.jar:dt29/PrivateFieldFamily.class */
public class PrivateFieldFamily {

    /* JADX INFO: loaded from: private-field-input.jar:dt29/PrivateFieldFamily$A.class */
    public static class A {
        public boolean visible;
        private boolean hidden;
    }

    /* JADX INFO: loaded from: private-field-input.jar:dt29/PrivateFieldFamily$B.class */
    public static class B extends A {
        public boolean visible;

        public void set(boolean z, boolean z2) {
            super.visible = z;
            ((A) this).hidden = z2;
        }
    }
}
