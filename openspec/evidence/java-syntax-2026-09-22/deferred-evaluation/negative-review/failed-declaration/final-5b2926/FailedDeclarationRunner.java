public class FailedDeclarationRunner {
    private static void one(int input, int mode) {
        Support.trace = 0;
        Support.mode = mode;
        try {
            int result = FailedDeclaration.run(input);
            System.out.println(input + ":" + mode + ":value=" + result + ":trace=" + Support.trace);
        } catch (RuntimeException error) {
            System.out.println(input + ":" + mode + ":error=" + error.getClass().getName()
                    + ":same=" + (error == Support.FAILURE)
                    + ":trace=" + Support.trace);
        }
    }

    public static void main(String[] args) {
        for (int input : new int[] {5, -3}) {
            for (int mode = 0; mode < 3; mode++) {
                one(input, mode);
            }
        }
    }
}
