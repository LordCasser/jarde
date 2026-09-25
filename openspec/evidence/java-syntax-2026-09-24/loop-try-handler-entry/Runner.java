public class Runner {
    public static void main(String[] args) {
        System.out.println("normal=" + LoopTryHandlerEntry.loopTry(3, -1));
        System.out.println("caught=" + LoopTryHandlerEntry.loopTry(3, 2));
    }
}
