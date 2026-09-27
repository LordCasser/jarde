public class IdentityRunner {
    public static void main(String[] args) {
        try {
            IdentityOnce.escaping();
            System.out.println("returned");
        } catch (Throwable error) {
            System.out.println((error == IdentityOnce.ORIGINAL) + ":" + error.getClass().getName() + ":" + error.getMessage() + ":" + IdentityOnce.count());
        }
    }
}
