public class SwitchDeferredCore {
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

}
