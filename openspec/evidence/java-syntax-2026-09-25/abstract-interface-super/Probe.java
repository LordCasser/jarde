public final class Probe implements Child {
    @Override public int value() { return Child.super.value(); }
    public static void main(String[] args) {
        System.out.println(new Probe().value());
    }
}
