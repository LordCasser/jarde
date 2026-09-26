public class Outer extends Base {
    private int value = 7;
    private String label() { return "outer"; }

    public class Member extends MemberBase {
        public int outerField() { return Outer.this.value; }
        public String outerMethod() { return Outer.this.label(); }
        public String outerSuper() { return Outer.super.value(); }
        public String ordinaryThis() { return this.value(); }
    }

    @Override public String value() { return "outer-override"; }
    public static void main(String[] args) {
        Outer outer = new Outer();
        Member member = outer.new Member();
        System.out.println(member.outerField() + ":" + member.outerMethod() + ":" + member.outerSuper() + ":" + member.ordinaryThis());
    }
}
