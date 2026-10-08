public class FunctionalConstructors extends java.lang.Object {
    public static int trace;

    public FunctionalConstructors() {
        super();
        return;
    }

    public static java.lang.Thread runnable() {
        return new java.lang.Thread((java.lang.Runnable) (() -> FunctionalConstructors.lambda$runnable$0$jarde()));
    }

    public static java.lang.Thread captured(int arg0) {
        return new java.lang.Thread((java.lang.Runnable) (() -> FunctionalConstructors.lambda$captured$1$jarde(arg0)));
    }

    public static java.util.PriorityQueue comparator() {
        return new java.util.PriorityQueue((java.util.Comparator) ((java.lang.Object arg0, java.lang.Object arg1) -> ((java.lang.Integer) arg1).intValue() - ((java.lang.Integer) arg0).intValue()));
    }

    public static java.util.PriorityQueue reference() {
        return new java.util.PriorityQueue((java.util.Comparator) ((java.lang.Object p0, java.lang.Object p1) -> FunctionalConstructors.compare((java.lang.Integer) p0, (java.lang.Integer) p1)));
    }

    public static int compare(java.lang.Integer arg0, java.lang.Integer arg1) {
        return arg1.intValue() - arg0.intValue();
    }

    public static java.util.concurrent.FutureTask callable() {
        return new java.util.concurrent.FutureTask((java.util.concurrent.Callable) (() -> 7));
    }

    public static IntBox primitive(int arg0) {
        return new IntBox((java.util.function.IntUnaryOperator) ((int arg1) -> arg0 + arg1));
    }

    public static IntBox primitiveReference() {
        return new IntBox((java.util.function.IntUnaryOperator) FunctionalConstructors::twice);
    }

    public static int twice(int arg0) {
        return arg0 * 2;
    }

    public static java.lang.String name() {
        FunctionalConstructors.trace = FunctionalConstructors.trace * 10 + 1;
        return "worker";
    }

    public static java.lang.Thread ordered(int arg0) {
        return new java.lang.Thread((java.lang.Runnable) (() -> FunctionalConstructors.lambda$ordered$5$jarde(arg0)), (java.lang.String) name());
    }

    public static java.lang.Thread overload() {
        return new java.lang.Thread((java.lang.Runnable) (() -> FunctionalConstructors.lambda$overload$6$jarde()));
    }

    private static void lambda$overload$6$jarde() {
        FunctionalConstructors.trace = FunctionalConstructors.trace + 2;
        return;
    }

    private static void lambda$ordered$5$jarde(int arg0) {
        FunctionalConstructors.trace = FunctionalConstructors.trace * 10 + arg0;
        return;
    }

    private static void lambda$captured$1$jarde(int arg0) {
        FunctionalConstructors.trace = FunctionalConstructors.trace + arg0;
        return;
    }

    private static void lambda$runnable$0$jarde() {
        FunctionalConstructors.trace = FunctionalConstructors.trace + 3;
        return;
    }
}