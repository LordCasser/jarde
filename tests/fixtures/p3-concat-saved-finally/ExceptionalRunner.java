public class ExceptionalRunner {
    public static void main(String[] args) {
        try {
            FinallyOnce.handled(true);
            System.out.println("returned");
        } catch (Throwable error) {
            System.out.println((error == ThrowingArgument.ORIGINAL) + ":" + error.getClass().getName() + ":" + error.getMessage() + ":" + FinallyOnce.count());
        }
    }
}
