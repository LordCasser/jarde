package cf02;

public class Runner {
    public static void main(String[] args) {
        Predicates test = new Predicates();
        System.out.println(test.greater(Float.NaN, 1.0f));
        System.out.println(test.greater(Float.POSITIVE_INFINITY, 1.0f));
        System.out.println(test.mixed(-0.0f, 0.0d));
        System.out.println(test.inBounds(new int[] {1, 2}, -1));
        System.out.println(test.inBounds(new int[] {1, 2}, 1));
        System.out.println(test.inBounds(new int[] {1, 2}, 2));
        System.out.println(test.named(null));
        System.out.println(test.named(Integer.valueOf(1)));
        System.out.println(test.named(""));
        System.out.println(test.named("x"));
    }
}
