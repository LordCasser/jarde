package defpackage;

/* JADX INFO: loaded from: Runner.class */
public final class Runner {
    private Runner() {
    }

    public static void main(String[] strArr) {
        for (int i : new int[]{9, 10, 11, 99, 100, 101}) {
            System.out.println("nested(" + i + ")=" + NestedConditional.nested(i));
        }
        for (int i2 : new int[]{0, 5, 10, 11, 12, 50, 100, 101, 150}) {
            StringBuilder sb = new StringBuilder();
            try {
                System.out.println("effects(" + i2 + ")=" + NestedConditional.nestedEffects(i2, sb) + ";events=" + ((Object) sb));
            } catch (RuntimeException e) {
                System.out.println("effects(" + i2 + ")=" + e.getClass().getSimpleName() + ";message=" + e.getMessage() + ";events=" + ((Object) sb));
            }
        }
    }
}
