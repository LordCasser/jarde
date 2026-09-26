package defpackage;

import java.util.Objects;

/* JADX INFO: loaded from: fixture.jar:Outer.class */
public class Outer extends Base {
    private int value = 7;

    /* JADX INFO: Access modifiers changed from: private */
    public String label() {
        return "outer";
    }

    /* JADX INFO: loaded from: fixture.jar:Outer$Member.class */
    public class Member extends MemberBase {
        public Member() {
        }

        public int outerField() {
            return Outer.this.value;
        }

        public String outerMethod() {
            return Outer.this.label();
        }

        public String outerSuper() {
            return Outer.super.value();
        }

        public String ordinaryThis() {
            return value();
        }
    }

    @Override // defpackage.Base
    public String value() {
        return "outer-override";
    }

    public static void main(String[] args) {
        Outer outer = new Outer();
        Objects.requireNonNull(outer);
        Member member = outer.new Member();
        System.out.println(member.outerField() + ":" + member.outerMethod() + ":" + member.outerSuper() + ":" + member.ordinaryThis());
    }
}
