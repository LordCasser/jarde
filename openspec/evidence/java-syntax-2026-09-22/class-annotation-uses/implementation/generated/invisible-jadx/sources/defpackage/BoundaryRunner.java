package defpackage;

/* JADX INFO: loaded from: BoundaryRunner.class */
public final class BoundaryRunner {
    public static void main(String[] strArr) {
        System.out.println(HiddenTarget.class.getDeclaredAnnotations().length);
        System.out.println(HiddenTarget.class.getAnnotation(HiddenTag.class));
    }
}
