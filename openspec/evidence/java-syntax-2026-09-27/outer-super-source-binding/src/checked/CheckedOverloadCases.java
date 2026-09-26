package checked;

import java.io.IOException;

class CheckedParent {
    String select(Number value) {
        return "no-checked-exception";
    }

    String select(Integer value) throws IOException {
        return "checked-exception";
    }
}

public class CheckedOverloadCases extends CheckedParent {
    class Member {
        String numberStaticType(Number value) {
            return CheckedOverloadCases.super.select(value);
        }
    }

    public static void main(String[] args) {
        CheckedOverloadCases outer = new CheckedOverloadCases();
        System.out.println(outer.new Member().numberStaticType(Integer.valueOf(3)));
    }
}
