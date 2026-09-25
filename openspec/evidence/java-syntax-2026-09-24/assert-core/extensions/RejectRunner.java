public class RejectRunner {
    public static void main(String[] args) {
        try {
            AssertExtraFieldAccess.check(false);
        } catch (AssertionError error) {
            System.out.print("extra=" + error.getClass().getSimpleName() + ":"
                    + error.getMessage() + ":writes=" + AssertExtraFieldAccess.extraWrites + ";");
        }
        System.out.print("probe=" + AssertExtraFieldAccess.statusProbe() + ";");
        try {
            AssertDifferentConstructor.check(false);
        } catch (AssertionError error) {
            System.out.print("ctor=" + error.getClass().getSimpleName() + ":"
                    + error.getMessage() + ";");
        }
        System.out.println();
    }
}
