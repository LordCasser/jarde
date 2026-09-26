package defpackage;

/* JADX INFO: loaded from: input.jar:AnonymousInterfaceBasic.class */
public class AnonymousInterfaceBasic {
    static I make() {
        return new I() { // from class: AnonymousInterfaceBasic.1
            @Override // defpackage.I
            public int value() {
                return 7;
            }
        };
    }

    public static void main(String[] strArr) {
        System.out.println(make().value());
    }
}
