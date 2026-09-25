public class Locked {
    int n;

    int locked() {
        synchronized (this) {
            return n;
        }
    }
}
