public class FC extends java.lang.Object {
    public FC() {
        super();
        return;
    }

    static java.lang.String compose() {
        java.util.function.Function local0 = (java.lang.Object arg0) -> java.lang.Integer.valueOf(((java.lang.Integer) arg0).intValue() * 2);
        java.util.function.Function local1 = (java.lang.Object arg0) -> java.lang.Integer.valueOf(((java.lang.Integer) arg0).intValue() + 1);
        java.util.function.Function local2 = local0.compose(local1);
        java.util.function.Function local3 = local0.andThen(local1);
        return "" + local2.apply((java.lang.Object) java.lang.Integer.valueOf(5)) + "/" + local3.apply((java.lang.Object) java.lang.Integer.valueOf(5));
    }

    static java.lang.String predicates(java.lang.String arg0) {
        java.util.function.Predicate local1 = (java.lang.Object p0) -> !((java.lang.String) p0).isEmpty();
        java.util.function.Predicate local2 = (java.lang.Object p0_) -> ((java.lang.String) p0_).length() < 5;
        java.util.function.Predicate local3 = local1.and(local2).negate();
        return "" + local3.test((java.lang.Object) arg0) + "/" + local1.or(local2).test((java.lang.Object) "");
    }

    static java.lang.String bi() {
        java.util.function.BiFunction local0 = (java.lang.Object arg0, java.lang.Object arg1) -> java.lang.Integer.valueOf(((java.lang.Integer) arg0).intValue() + ((java.lang.Integer) arg1).intValue());
        java.util.function.Supplier local1 = () -> "v" + local0.apply((java.lang.Object) java.lang.Integer.valueOf(2), (java.lang.Object) java.lang.Integer.valueOf(3));
        return (java.lang.String) local1.get();
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println((java.lang.String) compose());
        java.lang.System.out.println((java.lang.String) predicates("hello"));
        java.lang.System.out.println((java.lang.String) bi());
        return;
    }
}
