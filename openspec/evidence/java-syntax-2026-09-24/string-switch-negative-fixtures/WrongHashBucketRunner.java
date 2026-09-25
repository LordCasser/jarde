public class WrongHashBucketRunner {
    public static void main(String[] args) {
        String[] values = {"Aa", "BB", "abc", "x", null};
        for (String value : values) {
            WrongHashBucket.calls = 0;
            try {
                System.out.println(value + ":" + WrongHashBucket.choose(value)
                        + ":calls=" + WrongHashBucket.calls);
            } catch (RuntimeException error) {
                System.out.println(value + ":throws=" + error.getClass().getSimpleName()
                        + ":calls=" + WrongHashBucket.calls);
            }
        }
    }
}
