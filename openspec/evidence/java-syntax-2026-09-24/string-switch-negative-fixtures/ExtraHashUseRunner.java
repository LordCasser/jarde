public class ExtraHashUseRunner {
    public static void main(String[] args) {
        String[] values = {"Aa", "BB", "abc", "x", "a", null};
        for (String value : values) {
            ExtraHashUse.calls = 0;
            try {
                System.out.println(value + ":" + ExtraHashUse.choose(value)
                        + ":calls=" + ExtraHashUse.calls);
            } catch (RuntimeException error) {
                System.out.println(value + ":throws=" + error.getClass().getSimpleName()
                        + ":calls=" + ExtraHashUse.calls);
            }
        }
    }
}
