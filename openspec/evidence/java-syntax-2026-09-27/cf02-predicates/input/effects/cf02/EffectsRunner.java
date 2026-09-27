package cf02;

public class EffectsRunner {
    public static void main(String[] args) {
        Object[] values = {null, Integer.valueOf(1), "", "x"};
        for (Object value : values) {
            PredicateEffects.calls = 0;
            System.out.println(PredicateEffects.named(value) + ":" + PredicateEffects.calls);
        }
    }
}
