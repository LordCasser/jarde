package dt24;

public class Runner {
    public static void main(String[] args) {
        Mix value = Tagged.class.getAnnotation(Mix.class);
        System.out.println(value.name() + ":" + value.num() + ":" + value.value()
                + ":" + java.util.Arrays.toString(value.doubles())
                + ":" + value.cls().getName() + ":" + value.mode()
                + ":" + value.nested().value()
                + ":" + java.util.Arrays.toString(value.ints()));
    }
}
