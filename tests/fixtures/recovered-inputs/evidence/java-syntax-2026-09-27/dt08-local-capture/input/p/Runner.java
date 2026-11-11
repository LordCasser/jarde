package p;

public class Runner {
    public static void main(String[] args) {
        Runnable first = Capture.create(2.5);
        Runnable second = Capture.create(-0.0);
        first.run();
        second.run();
    }
}
