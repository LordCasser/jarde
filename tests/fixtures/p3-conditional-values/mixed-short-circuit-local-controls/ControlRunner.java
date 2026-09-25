public final class ControlRunner {
    public static void main(String[] args) {
        MixedLocalControls.bValue = false;
        MixedLocalControls.cValue = true;
        System.out.println(MixedLocalControls.numeric(true));
        System.out.println(MixedLocalControls.rewritten(true));
        System.out.println(MixedLocalControls.duplicated(true));
        System.out.println(MixedLocalControls.crossing(true));
    }
}
