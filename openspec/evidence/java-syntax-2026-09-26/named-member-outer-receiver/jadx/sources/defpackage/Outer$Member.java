package defpackage;

/* JADX INFO: loaded from: Outer$Member.class */
public class Outer$Member extends MemberBase {
    final /* synthetic */ Outer this$0;

    public Outer$Member(Outer this$0) {
        this.this$0 = this$0;
    }

    public int outerField() {
        return Outer.access$000(this.this$0);
    }

    public String outerMethod() {
        return Outer.access$100(this.this$0);
    }

    public String outerSuper() {
        return Outer.access$201(this.this$0);
    }

    public String ordinaryThis() {
        return value();
    }
}
