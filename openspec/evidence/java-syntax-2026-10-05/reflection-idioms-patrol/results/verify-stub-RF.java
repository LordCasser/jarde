public class RF extends java.lang.Object {
    public RF() {
        super();
        return;
    }

    static java.lang.Object load(java.lang.String arg0) throws java.lang.Exception {
        return java.lang.Class.forName(arg0).newInstance();
    }

    static java.lang.Object call(java.lang.Object arg0, java.lang.String arg1, java.lang.Object arg2) throws java.lang.Exception {
        java.lang.Class saved0 = arg0.getClass();
        java.lang.reflect.Method saved1 = saved0.getMethod(arg1, new java.lang.Class[]{java.lang.String.class});
        return saved1.invoke(arg0, new java.lang.Object[]{arg2});
    }

    static boolean kind(java.lang.Object arg0, java.lang.String arg1) throws Exception { return true; }
    static java.lang.String ctor(java.lang.String arg0, java.lang.String arg1) throws java.lang.Exception {
        java.lang.Class saved0 = java.lang.Class.forName(arg0);
        java.lang.reflect.Constructor saved1 = saved0.getConstructor(new java.lang.Class[]{java.lang.String.class});
        return (java.lang.String) saved1.newInstance(new java.lang.Object[]{arg1});
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("").append(load("java.lang.StringBuilder") instanceof java.lang.StringBuilder).append("/").append((java.lang.Object) call((java.lang.Object) new java.lang.StringBuilder("ab"), "append", (java.lang.Object) "c")).append("/").append(kind((java.lang.Object) "xy", "java.lang.String")).append("/").append((java.lang.String) ctor("java.lang.String", "made")).toString());
        return;
    }
}
