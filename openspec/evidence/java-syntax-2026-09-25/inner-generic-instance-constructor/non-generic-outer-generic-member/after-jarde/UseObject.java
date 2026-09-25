package minimal;

public final class UseObject {
    private static int argumentCalls;

    private static int argument(int value) {
        argumentCalls++;
        return value;
    }

    public static java.lang.Object make(minimal.Outer arg0, int arg1) {
        // @method make(Lminimal/Outer;I)Ljava/lang/Object;
        // @declaration a static method of `minimal.UseObject`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.new Inner<>((java.lang.Object) java.lang.Integer.valueOf(argument(arg1)));
    }


    public static void main(String[] args) {
        Object result = make(new Outer(), 7);
        System.out.println(result.getClass().getName() + ":" + argumentCalls);
        try {
            make(null, 9);
        } catch (NullPointerException expected) {
            System.out.println("null:" + argumentCalls);
        }
    }
}
