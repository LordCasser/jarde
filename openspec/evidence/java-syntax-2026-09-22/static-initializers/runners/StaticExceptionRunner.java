public class StaticExceptionRunner {
    public static void main(String[] args) {
        try {
            System.out.println("value=" + StaticException.get());
        } catch (Throwable error) {
            System.out.println(error.getClass().getName() + ":" + error.getCause().getClass().getName());
        }
    }
}
