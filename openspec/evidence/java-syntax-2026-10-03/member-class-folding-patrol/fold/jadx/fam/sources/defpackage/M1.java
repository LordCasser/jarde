package defpackage;

/* JADX INFO: loaded from: fam.jar:M1.class */
public class M1 {
    static Deep deepField;
    Base baseField;

    /* JADX INFO: loaded from: fam.jar:M1$Base.class */
    static class Base {
        Base() {
        }

        void hi() {
            System.out.println("hi");
        }
    }

    /* JADX INFO: loaded from: fam.jar:M1$Ctrl.class */
    interface Ctrl {
    }

    /* JADX INFO: loaded from: fam.jar:M1$Deep.class */
    static class Deep extends Base implements Ctrl {
        Deep() {
        }
    }

    /* JADX INFO: loaded from: fam.jar:M1$Err.class */
    static class Err extends Exception {
        Err() {
        }
    }

    /* JADX INFO: loaded from: fam.jar:M1$Inner.class */
    static class Inner {

        /* JADX INFO: loaded from: fam.jar:M1$Inner$Leaf.class */
        static class Leaf extends Base {
            Leaf() {
            }
        }

        Inner() {
        }
    }

    void work() throws RuntimeException, Err {
    }

    static Deep make() throws Err {
        return new Deep();
    }

    public static void main(String[] strArr) throws Err {
        M1 m1 = new M1();
        m1.baseField = new Deep();
        m1.baseField.hi();
        m1.work();
        System.out.println("ok");
    }
}
