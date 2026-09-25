package nested;

/* JADX INFO: loaded from: full-target.jar:nested/MissingRunner.class */
public final class MissingRunner {
    public static void main(String[] strArr) {
        try {
            System.out.println("made:" + UseInner.make(new SimpleOuter(4), 3).getClass().getName());
        } catch (NoClassDefFoundError e) {
            System.out.println("missing:" + e.getMessage());
        }
    }
}
