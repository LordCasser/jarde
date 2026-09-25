package defpackage;

/* JADX INFO: loaded from: BoundaryRunner.class */
public final class BoundaryRunner {
    public static void main(String[] strArr) throws Exception {
        BoundaryTagged boundaryTagged = new BoundaryTagged();
        System.out.println(boundaryTagged.wideAndVarargs(2L, 3.0d, "x", "yy"));
        System.out.println(boundaryTagged.sameTypeAtDistinctPositions(4, "abc"));
        System.out.println(BoundaryTagged.class.getDeclaredField("field").getDeclaredAnnotations().length);
        System.out.println(BoundaryTagged.class.getDeclaredMethod("wideAndVarargs", Long.TYPE, Double.TYPE, String[].class).getParameterAnnotations().length);
    }
}
