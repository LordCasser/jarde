public class PositiveDriver {
    public static void main(String[] args) {
        System.out.println(new KnownBoundPositive().entryThis().get());
        System.out.println(KnownBoundPositive.constantString().getAsInt());
    }
}
