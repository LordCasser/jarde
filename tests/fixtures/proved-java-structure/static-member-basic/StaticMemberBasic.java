public class StaticMemberBasic {
    static class Leaf {
        int value() {
            return 9;
        }
    }

    static Leaf make() {
        return new Leaf();
    }

    public static void main(String[] args) {
        System.out.println(make().value() + ":" + Named$Top.value());
    }
}
