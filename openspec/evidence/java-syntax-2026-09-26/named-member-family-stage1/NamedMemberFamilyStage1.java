public class NamedMemberFamilyStage1 {
    int state;
    private int secret;

    NamedMemberFamilyStage1(int state, int secret) {
        this.state = state;
        this.secret = secret;
    }

    class Member {
        int read(NamedMemberFamilyStage1 other) {
            return other.state * 100 + NamedMemberFamilyStage1.this.secret;
        }
    }

    public static void main(String[] args) {
        NamedMemberFamilyStage1 outer = new NamedMemberFamilyStage1(10, 11);
        NamedMemberFamilyStage1 other = new NamedMemberFamilyStage1(20, 22);
        System.out.println(outer.new Member().read(other));
        System.out.println(other.state);
    }
}
