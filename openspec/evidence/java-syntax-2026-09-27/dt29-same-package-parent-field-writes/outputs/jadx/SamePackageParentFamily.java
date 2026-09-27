package dt29;

/* JADX INFO: loaded from: fixture.jar:dt29/SamePackageParentFamily.class */
public class SamePackageParentFamily {

    /* JADX INFO: loaded from: fixture.jar:dt29/SamePackageParentFamily$A.class */
    public static class A {
        public boolean publicField;
        protected boolean protectedField;
        boolean packagePrivateField;
        private boolean privateField;
        public static boolean staticField;
        int descriptorControl;
    }

    /* JADX INFO: loaded from: fixture.jar:dt29/SamePackageParentFamily$B.class */
    public static class B extends A {
        protected boolean protectedField;
        boolean packagePrivateField;

        public void set(boolean z, boolean z2) {
            this.publicField = z;
            super.protectedField = z;
            super.packagePrivateField = z;
            ((A) this).privateField = z2;
        }
    }
}
