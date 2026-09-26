package generic;

import java.io.PrintStream;
import java.util.Objects;

/* JADX INFO: loaded from: GenericRawCases.class */
class GenericRawCases extends GenericParent {

    /* JADX INFO: loaded from: GenericRawCases$Member.class */
    class Member {
        Member() {
        }

        String choose(String str) {
            return GenericRawCases.super.choose((CharSequence) str);
        }
    }

    GenericRawCases() {
    }

    public static void main(String[] strArr) {
        GenericRawCases genericRawCases = new GenericRawCases();
        PrintStream printStream = System.out;
        Objects.requireNonNull(genericRawCases);
        printStream.println(genericRawCases.new Member().choose("value"));
    }
}
