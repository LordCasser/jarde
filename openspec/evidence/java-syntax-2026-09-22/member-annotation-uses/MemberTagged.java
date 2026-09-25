public class MemberTagged {
    @Deprecated
    public int field = 3;

    @Deprecated
    public int value(@Deprecated int x) {
        return x + field;
    }
}
