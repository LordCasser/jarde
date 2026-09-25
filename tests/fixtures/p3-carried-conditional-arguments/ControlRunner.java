public final class ControlRunner {
    public static void main(String[] args) {
        for (int mode : new int[] {0, 1, 2}) {
            System.out.println(new CarriedTypeMismatch("in", mode == 1, mode).values());
            System.out.println(new CarriedThirdArgument("in", mode).values());
            System.out.println(new CarriedInterveningEffect("in", mode).values());
            System.out.println(new CarriedNonPrologue("in", mode).values());
        }
        for (boolean first : new boolean[] {false, true}) {
            for (boolean second : new boolean[] {false, true}) {
                System.out.println(CarriedMethodCalls.staticCall(first, second));
                System.out.println(new CarriedMethodCalls().instanceCall(first, second));
            }
        }
    }
}
