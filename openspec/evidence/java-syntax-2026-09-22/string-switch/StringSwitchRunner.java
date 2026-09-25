public class StringSwitchRunner {
    public static void main(String[] args) {
        for (String value : new String[] {"Aa", "BB", "z", "other", null}) {
            StringSwitchProbe.calls = 0;
            try {
                System.out.println("choose:" + value + ":" + StringSwitchProbe.choose(value));
            } catch (Throwable failure) {
                System.out.println("choose:" + value + ":" + failure.getClass().getName());
            }
            try {
                System.out.println("once:" + value + ":" + StringSwitchProbe.once(value)
                        + ":calls=" + StringSwitchProbe.calls);
            } catch (Throwable failure) {
                System.out.println("once:" + value + ":" + failure.getClass().getName()
                        + ":calls=" + StringSwitchProbe.calls);
            }
        }
    }
}
