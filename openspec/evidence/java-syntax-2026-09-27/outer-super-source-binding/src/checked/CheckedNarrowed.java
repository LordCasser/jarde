package checked;

public class CheckedNarrowed extends CheckedParent {
    class Member {
        String integerStaticType(Integer value) {
            return CheckedNarrowed.super.select(value);
        }
    }
}
