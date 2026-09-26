package checked;

import java.io.PrintStream;
import java.util.Objects;

/* JADX INFO: loaded from: CheckedOverloadCases.class */
public class CheckedOverloadCases extends CheckedParent {

    /* JADX INFO: loaded from: CheckedOverloadCases$Member.class */
    class Member {
        Member() {
        }

        String numberStaticType(Number number) {
            return CheckedOverloadCases.super.select(number);
        }
    }

    public static void main(String[] strArr) {
        CheckedOverloadCases checkedOverloadCases = new CheckedOverloadCases();
        PrintStream printStream = System.out;
        Objects.requireNonNull(checkedOverloadCases);
        printStream.println(checkedOverloadCases.new Member().numberStaticType(3));
    }
}
