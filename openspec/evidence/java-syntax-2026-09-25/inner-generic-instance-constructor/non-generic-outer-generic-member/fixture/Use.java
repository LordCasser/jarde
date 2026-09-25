package minimal;

public final class Use {
    public static Outer.Inner<Integer> make(Outer outer, int value) {
        return outer.new Inner<>(value);
    }

    public static void main(String[] args) {
        System.out.println(make(new Outer(), 7).value());
    }
}
