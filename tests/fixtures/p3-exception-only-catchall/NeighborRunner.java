public class NeighborRunner {
    public static void main(String[] args) {
        for (int i = 0; i < 2; i++) {
            try {
                FinallyOnce.escaping();
                System.out.println("returned:" + FinallyOnce.count());
            } catch (Throwable error) {
                System.out.println(error.getClass().getName() + ":" + error.getMessage() + ":" + FinallyOnce.count());
            }
        }
    }
}
