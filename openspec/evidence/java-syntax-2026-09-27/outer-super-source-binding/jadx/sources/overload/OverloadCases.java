package overload;

import java.util.Objects;

/* JADX INFO: loaded from: OverloadCases.class */
public class OverloadCases extends OverloadParent {

    /* JADX INFO: loaded from: OverloadCases$Member.class */
    class Member {
        Member() {
        }

        String numberStaticType(Number number) {
            return OverloadCases.super.select(number);
        }

        String integerStaticType(Integer num) {
            return OverloadCases.super.select(num);
        }
    }

    public static void main(String[] strArr) {
        OverloadCases overloadCases = new OverloadCases();
        Objects.requireNonNull(overloadCases);
        Member member = overloadCases.new Member();
        System.out.println(member.numberStaticType(7));
        System.out.println(member.integerStaticType(7));
    }
}
