public final class Main {
    public static void main(String[] args) {
        Base one = Owner.one();
        Base two = Other.two();
        System.out.println("sameClass=" + (one.getClass() == two.getClass()));
    }
}
