public class AL extends java.lang.Object {
    public AL() {
        super();
        return;
    }

    static java.util.List bridge(java.lang.String[] arg0) {
        return java.util.Arrays.asList((java.lang.Object[]) arg0);
    }

    static java.lang.String[] back(java.util.List arg0) {
        return (java.lang.String[]) arg0.toArray((java.lang.Object[]) new java.lang.String[0]);
    }

    static java.util.List frozen(java.util.List arg0) {
        return java.util.Collections.unmodifiableList(arg0);
    }

    static java.lang.String firstOr(java.util.List arg0, java.lang.String arg1) {
        return arg0.isEmpty() ? arg1 : (java.lang.String) arg0.get(0);
    }

    public static void main(java.lang.String[] arg0) {
        java.util.List local1 = bridge(new java.lang.String[]{"x", "y"});
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("").append((java.lang.Object) local1).append("/").append((java.lang.String) java.util.Arrays.toString((java.lang.Object[]) back(local1))).append("/").append(frozen(local1).size()).append("/").append((java.lang.String) firstOr((java.util.List) new java.util.ArrayList(), "E")).append("/").append((java.lang.String) firstOr(local1, "E")).toString());
        return;
    }
}
