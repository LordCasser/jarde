public class SynchronizedThreeWay {
    static int choose(Object lock, int selector) {
        synchronized (lock) {
            if (selector == 0) return 10;
            if (selector == 1) return 20;
            return 30;
        }
    }
    public static void main(String[] args) {
        Object lock = new Object();
        if (choose(lock, 0) != 10 || choose(lock, 1) != 20 || choose(lock, 2) != 30) throw new AssertionError();
        System.out.println("three-way:ok");
    }
}
