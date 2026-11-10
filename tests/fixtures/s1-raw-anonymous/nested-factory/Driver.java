package p;

public final class Driver {
    public static void main(String[] args) {
        Nested.create().make().run();
        System.out.println(Nested.trace);
    }
}
