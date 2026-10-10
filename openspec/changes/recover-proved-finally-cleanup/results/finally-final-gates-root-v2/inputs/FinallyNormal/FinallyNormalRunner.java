public class FinallyNormalRunner {
    public static void main(String[] args) {
        FinallyNormal.failAtOne = false;
        FinallyNormal.trace = 0;
        try {
            System.out.println("normal:return:" + FinallyNormal.run() + ":" + FinallyNormal.trace);
        } catch (RuntimeException e) {
            System.out.println("normal:unexpected-throw:" + e + ":" + FinallyNormal.trace);
        }

        FinallyNormal.failAtOne = true;
        FinallyNormal.trace = 0;
        try {
            System.out.println("throw:unexpected-return:" + FinallyNormal.run() + ":" + FinallyNormal.trace);
        } catch (RuntimeException e) {
            System.out.println("throw:" + e.getClass().getName() + ":same=" + (e == FinallyNormal.FAILURE) + ":" + FinallyNormal.trace);
        }
    }
}
