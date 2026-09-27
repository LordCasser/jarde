package dt29p3;

public class BranchedBits {
    public static class A {
        public boolean publicField;
        protected boolean protectedField;
        boolean packagePrivateField;
        private boolean privateField;
    }

    public static class B extends A {
    }

    public static String bits(A a) {
        return (a.publicField ? "1" : "0")
                + (a.protectedField ? "1" : "0")
                + (a.packagePrivateField ? "1" : "0")
                + (a.privateField ? "1" : "0");
    }
}
