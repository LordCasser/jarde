public class StringSwitchVariantsRunner {
    public static void main(String[] args) {
        for (String value : new String[] {"Aa", "BB", "x", "y", "", "雪", "other", null, "!"}) {
            StringSwitchVariants.calls = 0;
            try {
                System.out.println("choose:" + value + ":" + StringSwitchVariants.choose(value)
                        + ":calls=" + StringSwitchVariants.calls);
            } catch (Throwable failure) {
                System.out.println("choose:" + value + ":" + failure.getClass().getName()
                        + ":calls=" + StringSwitchVariants.calls);
            }
            StringSwitchVariants.calls = 0;
            try {
                System.out.println("once:" + value + ":" + StringSwitchVariants.once(value)
                        + ":calls=" + StringSwitchVariants.calls);
            } catch (Throwable failure) {
                System.out.println("once:" + value + ":" + failure.getClass().getName()
                        + ":calls=" + StringSwitchVariants.calls);
            }
        }
    }
}
