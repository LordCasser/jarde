package dt27;

public class Runner {
    public static void main(String[] args) {
        int absolute = StaticRef.operator().applyAsInt(-2);
        int number = new InstanceRef(-3).supplier().getAsInt();
        String exception = ConstructorRef.maker().make("x").getClass().getSimpleName();
        System.out.println(absolute + ":" + number + ":" + exception);
    }
}
