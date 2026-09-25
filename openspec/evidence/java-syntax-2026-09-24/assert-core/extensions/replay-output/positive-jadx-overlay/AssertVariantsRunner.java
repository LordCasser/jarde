package defpackage;
public class AssertVariantsRunner {
    private static void run(String label, int value) {
        System.out.print(label + "=" + value + ";");
    }

    private static void caught(String label, Runnable action) {
        try {
            action.run();
            System.out.print(label + "=none;");
        } catch (AssertionError error) {
            System.out.print(label + "=" + error.getClass().getSimpleName() + ":"
                    + error.getMessage() + ";");
        }
    }

    public static void main(String[] args) {
        System.out.print("init=" + AssertVariants.effects + ";");
        run("plain-true", AssertVariants.noMessage(true));
        caught("plain-false", new Runnable() {
            public void run() { AssertVariants.noMessage(false); }
        });
        run("multiple-true", AssertVariants.multiple(true, true));
        caught("multiple-second-false", new Runnable() {
            public void run() { AssertVariants.multiple(true, false); }
        });
        System.out.println("effects=" + AssertVariants.effects);
    }
}
