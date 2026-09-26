package defpackage;
public class AssertHandlerBoundaryRunner {
    public static void main(String[] args) {
        try {
            AssertHandlerBoundary.check(false);
            System.out.print("generated=" + AssertHandlerBoundary.caught + ";");
            try {
                AssertHandlerBoundary.check(true);
                System.out.println("condition=none");
            } catch (AssertionError error) {
                System.out.println("condition=escaped:" + error.getMessage());
            }
        } catch (AssertionError error) {
            System.out.println("generated=escaped:" + error.getMessage());
        }
    }
}
