public class SwitchDeferred {
    public static int run(int tag) {
        int value;
        switch (tag) {
            case 1:
                value = SwitchSupport.value();
                break;
            default:
                value = SwitchSupport.other();
                break;
        }
        SwitchSupport.mark();
        return value;
    }

    // Keep every pool reference needed by the exact Code replacement.
    public static void keepPool() {
        SwitchSupport.value();
        SwitchSupport.mark();
        SwitchSupport.other();
    }
}
