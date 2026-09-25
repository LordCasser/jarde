// P3 4.4: one static call to another class, one to the class itself.
//
//     javac --release 8 -g:none -d v8 Calls.java
//
// `boxed` compiles to `invokestatic java/lang/Integer.valueOf`, whose pool owner is not
// this class: the call's text names the owner. `local` compiles to `invokestatic
// Calls.own`, whose pool owner **is** this class: the text stays unqualified.
public class Calls {
    static int own(int n) {
        return n;
    }

    static Integer boxed(int n) {
        return Integer.valueOf(n);
    }

    static int local(int n) {
        return own(n);
    }
}
