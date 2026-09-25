public class StringSwitchMiddleDefaultRunner {
    public static void main(String[] args) {
        for (String value : new String[] {"Aa", "BB", "", "雪", "prefix", "suffix", "other", null, "!"}) {
            StringSwitchMiddleDefault.calls = 0;
            try {
                System.out.println("choose:" + value + ":" + StringSwitchMiddleDefault.choose(value)
                        + ":calls=" + StringSwitchMiddleDefault.calls);
            } catch (Throwable failure) {
                System.out.println("choose:" + value + ":" + failure.getClass().getName()
                        + ":calls=" + StringSwitchMiddleDefault.calls);
            }
            StringSwitchMiddleDefault.calls = 0;
            try {
                System.out.println("once:" + value + ":" + StringSwitchMiddleDefault.once(value)
                        + ":calls=" + StringSwitchMiddleDefault.calls);
            } catch (Throwable failure) {
                System.out.println("once:" + value + ":" + failure.getClass().getName()
                        + ":calls=" + StringSwitchMiddleDefault.calls);
            }
        }
    }
}
