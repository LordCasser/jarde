package overload;

class OverloadAncestor {
    String select(Number value) {
        return "ancestor-number";
    }
}

class OverloadParent extends OverloadAncestor {
    String select(Integer value) {
        return "parent-integer";
    }
}

public class OverloadCases extends OverloadParent {
    class Member {
        String numberStaticType(Number value) {
            return OverloadCases.super.select(value);
        }

        String integerStaticType(Integer value) {
            return OverloadCases.super.select(value);
        }
    }

    public static void main(String[] args) {
        OverloadCases outer = new OverloadCases();
        Member member = outer.new Member();
        Integer runtimeInteger = Integer.valueOf(7);
        System.out.println(member.numberStaticType(runtimeInteger));
        System.out.println(member.integerStaticType(runtimeInteger));
    }
}
