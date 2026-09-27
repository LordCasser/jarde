import java.util.Arrays;

public class FinallyLoopRunner {
    public static void main(String[] args) {
        FinallyLoop loop = new FinallyLoop();
        FinallyLoop.fail = args.length > 0;
        try {
            loop.test(Arrays.asList("1", "2"));
            System.out.println("ok:" + loop.count());
        } catch (IllegalStateException ex) {
            System.out.println("throw:" + loop.count() + ":" + ex.getMessage());
        }
    }
}
