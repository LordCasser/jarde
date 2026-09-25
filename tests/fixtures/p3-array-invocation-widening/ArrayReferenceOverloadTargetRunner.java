public class ArrayReferenceOverloadTargetRunner {
    private interface Call {
        String run();
    }

    private static void call(String name, Call value) {
        try {
            System.out.println(name + "=" + value.run());
        } catch (Throwable error) {
            System.out.println(name + "=error:" + error.getClass().getName());
        }
    }

    public static void main(String[] args) {
        final String[] empty = new String[0];
        final String[] sized = new String[3];
        final String[] absent = null;
        call("local-empty", () -> ArrayReferenceOverloadTarget.localObjectTarget(empty));
        call("local-sized", () -> ArrayReferenceOverloadTarget.localObjectTarget(sized));
        call("implicit-empty", () -> ArrayReferenceOverloadTarget.implicitStringTarget(empty));
        call("implicit-sized", () -> ArrayReferenceOverloadTarget.implicitStringTarget(sized));
        call("local-null", () -> ArrayReferenceOverloadTarget.localObjectTarget(absent));
        call("implicit-null", () -> ArrayReferenceOverloadTarget.implicitStringTarget(absent));
        call("effectful", ArrayReferenceOverloadTarget::effectfulObjectTarget);
        System.out.println("effectful-evaluations=" + ArrayReferenceOverloadTarget.effectfulEvaluationCount());
    }
}
