public class AnonymousSuperDirect {
    private static int effects;

    private static int next() {
        effects++;
        return effects == 1 ? 7 : 2;
    }

    private static Base make() {
        return new Base(next(), next()) {
            @Override
            int sum() {
                return super.sum() + 1;
            }
        };
    }

    public static void main(String[] args) {
        System.out.println(make().sum() + ":" + effects);
    }
}

class Base {
    private final int left;
    private final int right;

    Base(int left, int right) {
        this.left = left;
        this.right = right;
    }

    Base(int left, long right) {
        this(left, (int) right);
    }

    int sum() {
        return left * 2 - right;
    }
}
