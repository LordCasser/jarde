package defpackage;

public class Runner {
    public static void main(String[] args) {
        if (DeepFinally.trace != 0) {
            throw new AssertionError("trace did not start at its default value");
        }
        int zero = DeepFinally.run(0);
        if (zero != 0 || DeepFinally.trace != 1) {
            throw new AssertionError("run(0): result=" + zero + " trace=" + DeepFinally.trace);
        }
        System.out.println("run(0)=" + zero + " trace=" + DeepFinally.trace);

        int positive = DeepFinally.run(40);
        if (positive != 1 || DeepFinally.trace != 2) {
            throw new AssertionError("run(40): result=" + positive + " trace=" + DeepFinally.trace);
        }
        System.out.println("run(40)=" + positive + " trace=" + DeepFinally.trace);
    }
}
