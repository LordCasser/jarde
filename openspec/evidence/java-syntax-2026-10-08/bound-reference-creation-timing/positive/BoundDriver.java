public class BoundDriver {
    public static void main(String[] args) {
        System.out.println(new KnownBound().entryThis().get());
        System.out.println(KnownBound.constantString().getAsInt());
        System.out.println(KnownBound.constantClass().get());
    }
}
