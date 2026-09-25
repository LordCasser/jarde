package negative;

public class CoreCaller {
    public static void invoke() {
        new GenericThrowsCore().<RuntimeException>run();
    }
}
