package defpackage;

/* JADX INFO: loaded from: MemberTagged.class */
public class MemberTagged {

    @Deprecated
    public int field = 3;

    @Deprecated
    public int value(@Deprecated int i) {
        return i + this.field;
    }
}
