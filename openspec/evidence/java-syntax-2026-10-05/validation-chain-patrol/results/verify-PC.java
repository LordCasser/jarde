public class PC extends java.lang.Object {
    public PC() {
        super();
        return;
    }

    static void check(java.lang.String arg0, int arg1, java.lang.Object arg2) {
        if (arg0 == null) {
            throw new java.lang.IllegalArgumentException("name is null");
        } else if (arg0.isEmpty()) {
            throw new java.lang.IllegalArgumentException("name is empty");
    } else if (arg1 < 0) {
            throw new java.lang.IllegalArgumentException("age: " + arg1);
    } else if (arg1 > 150) {
            throw new java.lang.IllegalArgumentException("age too big: " + arg1 + " for " + arg0);
    } else if (arg2 == null) {
            throw new java.lang.NullPointerException("extra required");
    } else {
            return;
    }
    }

    static java.lang.String fmt(java.lang.String arg0, java.lang.Object arg1) {
        return new java.lang.StringBuilder(arg0.length() + 16).append(arg0).append('=').append(arg1).toString();
    }

    public static void main(java.lang.String[] arg0) {
        try {
            check((java.lang.String) null, 1, new java.lang.Object());
        } catch (java.lang.IllegalArgumentException local1) {
            java.lang.System.out.println("A:" + local1.getMessage());
        }
        try {
            check("x", -1, new java.lang.Object());
        } catch (java.lang.IllegalArgumentException local1) {
            java.lang.System.out.println("B:" + local1.getMessage());
        }
        try {
            check("bob", 200, new java.lang.Object());
        } catch (java.lang.IllegalArgumentException local1) {
            java.lang.System.out.println("C:" + local1.getMessage());
        }
        try {
            check("ok", 1, (java.lang.Object) null);
        } catch (java.lang.NullPointerException local1) {
            java.lang.System.out.println("D:" + local1.getMessage());
        }
        java.lang.System.out.println("fmt:" + fmt("count", (java.lang.Object) java.lang.Integer.valueOf(42)));
        return;
    }
}
