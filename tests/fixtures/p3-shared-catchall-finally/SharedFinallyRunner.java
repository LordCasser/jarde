public class SharedFinallyRunner {
    public static void main(String[] args) {
        System.out.println(SharedFinally.handled(false) + ":" + SharedFinally.count());
        System.out.println(SharedFinally.handled(true) + ":" + SharedFinally.count());
    }
}
