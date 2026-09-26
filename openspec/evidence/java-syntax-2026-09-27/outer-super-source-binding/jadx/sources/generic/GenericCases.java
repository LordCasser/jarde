package generic;

import java.io.PrintStream;
import java.util.Objects;

/* JADX INFO: loaded from: GenericCases.class */
public class GenericCases extends GenericParent<String> {

    /* JADX INFO: loaded from: GenericCases$Member.class */
    class Member {
        Member() {
        }

        String choose(String str) {
            return GenericCases.super.choose(str);
        }
    }

    public static void main(String[] strArr) {
        GenericCases genericCases = new GenericCases();
        PrintStream printStream = System.out;
        Objects.requireNonNull(genericCases);
        printStream.println(genericCases.new Member().choose("value"));
    }
}
