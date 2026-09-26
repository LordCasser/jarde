public class AssertRunner {
    public static void main(String[] args) {
        System.out.print(AssertCore.check(true) + ";");
        try {
            System.out.print(AssertCore.check(false));
        } catch (AssertionError error) {
            System.out.print(error.getMessage() + "|" + AssertCore.counts());
        }
        System.out.println();
    }
}
