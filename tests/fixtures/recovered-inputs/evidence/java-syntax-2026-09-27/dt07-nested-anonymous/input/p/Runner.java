package p;

public final class Runner {
    public static void main(String[] args) {
        Nested.create().make().run();
        System.out.println(Nested.trace);
    }
}
