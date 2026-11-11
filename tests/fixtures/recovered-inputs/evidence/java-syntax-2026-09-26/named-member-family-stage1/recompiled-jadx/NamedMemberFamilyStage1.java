
import java.io.PrintStream;
import java.util.Objects;

/* JADX INFO: loaded from: fixture.jar:NamedMemberFamilyStage1.class */
public class NamedMemberFamilyStage1 {
    int state;
    private int secret;

    NamedMemberFamilyStage1(int state, int secret) {
        this.state = state;
        this.secret = secret;
    }

    /* JADX INFO: loaded from: fixture.jar:NamedMemberFamilyStage1$Member.class */
    class Member {
        Member() {
        }

        int read(NamedMemberFamilyStage1 other) {
            return (other.state * 100) + NamedMemberFamilyStage1.this.secret;
        }
    }

    public static void main(String[] args) {
        NamedMemberFamilyStage1 outer = new NamedMemberFamilyStage1(10, 11);
        NamedMemberFamilyStage1 other = new NamedMemberFamilyStage1(20, 22);
        PrintStream printStream = System.out;
        Objects.requireNonNull(outer);
        printStream.println(outer.new Member().read(other));
        System.out.println(other.state);
    }
}
