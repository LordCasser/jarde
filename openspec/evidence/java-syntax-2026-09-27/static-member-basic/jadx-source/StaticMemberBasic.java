package defpackage;

/* JADX INFO: loaded from: input.jar:StaticMemberBasic.class */
public class StaticMemberBasic {

    /* JADX INFO: loaded from: input.jar:StaticMemberBasic$Leaf.class */
    static class Leaf {
        Leaf() {
        }

        int value() {
            return 9;
        }
    }

    static Leaf make() {
        return new Leaf();
    }

    public static void main(String[] strArr) {
        System.out.println(make().value() + ":" + Named$Top.value());
    }
}
