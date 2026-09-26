package defpackage;
public class AssertDuplicateStatusWriteRunner {
    public static void main(String[] args) {
        try {
            AssertDuplicateStatusWrite.check(false);
            System.out.println("none");
        } catch (AssertionError error) {
            System.out.println("AssertionError");
        }
    }
}
