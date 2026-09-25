public class ConditionalBoundarySwitchRunner {
    public static void main(String[] args) {
        for (int selector : new int[] {0, 1, 2, -1}) {
            ConditionalBoundarySwitch.reset();
            System.out.println(selector + "=" + ConditionalBoundarySwitch.choose(selector)
                + ":trace=" + ConditionalBoundarySwitch.trace());
        }
    }
}
