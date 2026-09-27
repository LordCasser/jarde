package em12;

import java.io.PrintStream;
import java.util.Objects;

/* JADX INFO: loaded from: binding.jar:em12/Case.class */
public class Case extends Parent {

    /* JADX INFO: loaded from: binding.jar:em12/Case$Member.class */
    class Member {
        Member() {
        }

        String call(Arg arg) {
            return Case.super.pick(arg);
        }
    }

    public static void main(String[] strArr) {
        Case r0 = new Case();
        PrintStream printStream = System.out;
        Objects.requireNonNull(r0);
        Objects.requireNonNull(r0);
        printStream.println(r0.new Member().call(null));
    }
}
