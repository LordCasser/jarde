public class Runner {
    public static void main(String[] args) {
        System.out.println("normal=" + LoopTryHandlerEntryArgs.loopTry(3, -1, 0));
        System.out.println("caught=" + LoopTryHandlerEntryArgs.loopTry(3, 2, 0));
    }
}
