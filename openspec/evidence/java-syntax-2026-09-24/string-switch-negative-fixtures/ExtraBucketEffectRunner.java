public class ExtraBucketEffectRunner {
    public static void main(String[] args) {
        for (String value : new String[] { "Aa", "BB", "other", null }) {
            ExtraBucketEffect.calls = 0;
            try {
                System.out.println(value + ":" + ExtraBucketEffect.choose(value)
                        + ":calls=" + ExtraBucketEffect.calls);
            } catch (RuntimeException error) {
                System.out.println(value + ":" + error.getClass().getSimpleName()
                        + ":calls=" + ExtraBucketEffect.calls);
            }
        }
    }
}
