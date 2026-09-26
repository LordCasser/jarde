public class NamedMemberFamilyCalls {
    static final StringBuilder EVENTS = new StringBuilder();
    int state;

    NamedMemberFamilyCalls(int state) {
        this.state = state;
    }

    class Member {
        Member(int ignored) {}
        int read() { return NamedMemberFamilyCalls.this.state; }
    }

    static int mark(String label, int value) {
        EVENTS.append(label);
        return value;
    }

    Member internal(NamedMemberFamilyCalls outer, int value) {
        return outer.new Member(mark("I", value));
    }

    Member before(NamedMemberFamilyCalls outer, int value) {
        int prepared = mark("P", value);
        return outer.new Member(prepared);
    }

    public static void main(String[] args) {
        NamedMemberFamilyCalls owner = new NamedMemberFamilyCalls(7);
        try { owner.internal(null, 1); } catch (NullPointerException expected) {}
        try { owner.before(null, 2); } catch (NullPointerException expected) {}
        System.out.println(EVENTS.toString());
        System.out.println(owner.internal(owner, 3).read());
        System.out.println(owner.before(owner, 4).read());
        System.out.println(EVENTS.toString());
    }
}
