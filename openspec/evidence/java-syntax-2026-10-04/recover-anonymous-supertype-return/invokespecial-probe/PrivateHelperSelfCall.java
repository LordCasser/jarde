abstract class Base {
    abstract String render();
}

public final class PrivateHelperSelfCall {
    static Base create() {
        return new Base() {
            @Override
            String render() {
                return helper();
            }

            private String helper() {
                return "helper";
            }
        };
    }

    public static void main(String[] args) {
        System.out.println(create().render());
    }
}
