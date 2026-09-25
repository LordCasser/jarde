public class NegativeRunner {
    public static void main(String[] args) {
        String[] values = {"Aa", "BB", "abc", "x", null};
        for (String value : values) {
            ExtraHashUse.calls = 0;
            try {
                System.out.println("extra:" + value + ":" + ExtraHashUse.choose(value)
                        + ":calls=" + ExtraHashUse.calls);
            } catch (RuntimeException error) {
                System.out.println("extra:" + value + ":throws=" + error.getClass().getSimpleName()
                        + ":calls=" + ExtraHashUse.calls);
            }
            WrongHashBucket.calls = 0;
            try {
                System.out.println("wrong:" + value + ":" + WrongHashBucket.choose(value)
                        + ":calls=" + WrongHashBucket.calls);
            } catch (RuntimeException error) {
                System.out.println("wrong:" + value + ":throws=" + error.getClass().getSimpleName()
                        + ":calls=" + WrongHashBucket.calls);
            }
        }
    }
}
