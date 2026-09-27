package dt29p3;

public class BranchedBitsNegatives {
    public static class A {
        public boolean publicField;
        protected boolean protectedField;
        boolean packagePrivateField;
        private boolean privateField;
    }

    private static boolean touch(A a) {
        a.publicField = !a.publicField;
        return a.protectedField;
    }

    public static String alias(A a) {
        StringBuilder builder = new StringBuilder();
        StringBuilder alias = builder;
        return alias.append(a.publicField ? "1" : "0")
                .append(a.protectedField ? "1" : "0")
                .append(a.packagePrivateField ? "1" : "0")
                .append(a.privateField ? "1" : "0").toString();
    }

    public static String reused(A a) {
        String first = a.publicField ? "1" : "0";
        return first + (a.protectedField ? "1" : "0")
                + (a.packagePrivateField ? "1" : "0") + first;
    }

    public static String exchanged(A a) {
        String second = a.protectedField ? "1" : "0";
        String third = a.packagePrivateField ? "1" : "0";
        return (a.publicField ? "1" : "0")
                + third
                + second
                + (a.privateField ? "1" : "0");
    }

    public static String effect(A a) {
        return (a.publicField ? "1" : "0") + (touch(a) ? "1" : "0")
                + (a.packagePrivateField ? "1" : "0")
                + (a.privateField ? "1" : "0");
    }

    public static String handler(A a) {
        try {
            return (a.publicField ? "1" : "0")
                    + (a.protectedField ? "1" : "0")
                    + (a.packagePrivateField ? "1" : "0")
                    + (a.privateField ? "1" : "0");
        } catch (RuntimeException e) {
            return "caught";
        }
    }

    public static String overload(A a) {
        return new StringBuilder().append(a.publicField ? "1" : "0")
                .append((Object) (a.protectedField ? "1" : "0"))
                .append(a.packagePrivateField ? "1" : "0")
                .append(a.privateField ? "1" : "0").toString();
    }

    public static String missing(A a) {
        return (a.publicField ? "1" : "0")
                + (a.protectedField ? "1" : "0")
                + (a.packagePrivateField ? "1" : "0") + "0";
    }
}
